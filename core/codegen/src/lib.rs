extern crate proc_macro;

use aws_oxide_api_route::Route;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    AttributeArgs,
    Error as SynError,
    FnArg,
    GenericArgument,
    Ident,
    ItemFn,
    Lit,
    LitStr,
    NestedMeta,
    parse_macro_input,
    Pat,
    PathArguments,
    PatType,
    spanned::Spanned,
    Type,
};
use quote::{
    format_ident,
    ToTokens,
    quote,
    quote_spanned,
};

#[derive(Debug)]
struct ParameterType<'a> {
    param_type: &'a Ident,  // NOTE: Will eventually be used
    param_path: &'a syn::Path,
}


#[derive(Debug)]
struct Parameter<'a> {
    param_name: &'a Ident,
    param_type: ParameterType<'a>,
}


#[proc_macro_attribute]
pub fn route(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = syn::parse_macro_input!(item as ItemFn);
    let asyncness = &input.sig.asyncness;
    let attrs = &input.attrs;
    let ret = &input.sig.output;
    let fn_name = &input.sig.ident;
    let body = &input.block;
    let inputs = &input.sig.inputs;
    let mapping = Ident::new("mapping", Span::call_site());
    let fn_shim = format_ident!("{}_shim", fn_name);
    let fn_actual = format_ident!("{}_actual", fn_name);

    let mut param_expansion = Vec::new();
    let mut param_ident = Vec::new();

    // Parse out the arguments provided to the macro and validate
    // they form a proper route
    let (method, route) = match parse_route_arguments(&args) {
        Ok((method, route)) => {
            let validate = Route::validate(method.value(), route.value());

            if let Err(err) = validate {
                return TokenStream::from(
                    SynError::new(
                        input.span(),
                        err,
                    ).to_compile_error()
                )
            };

            (method, route)
        }
        Err(err) => {
            return TokenStream::from(
                SynError::new(
                    input.span(),
                    err,
                ).to_compile_error()
            )
        }
    };

    // For each parameter in the function signature parse and generate
    // code based on what it is.
    for fn_arg in inputs {
        match fn_arg {
            FnArg::Typed(pat_type) => {
                let parameter = if let Some(parameter) = extract_parameter(pat_type) {
                    parameter
                } else {
                    let tokens = quote_spanned! {
                        fn_arg.span() => compile_error!("Unable to extract parameter from function signature; unknown or unsupported argument type");
                    };

                    return TokenStream::from(tokens)
                };

                let param_match_expansion = parameter_match_expansion(
                    &mapping,
                    &parameter,
                );

                let (pname_v, tokens) = match param_match_expansion {
                    Ok((pname_v, tokens)) => {
                        (pname_v, tokens)
                    },
                    Err(err) => {
                        return TokenStream::from(
                            SynError::new(
                                input.span(),
                                err,
                            ).to_compile_error()
                        )
                    }
                };

                param_expansion.push(tokens);
                param_ident.push(pname_v);
            },
            _ => continue,
        };
    }

    let await_fn = if asyncness.is_some() {
        Some(quote! {.await})
    } else {
        None
    };

    // Code generation is a little convoluted; here's the TL;DR
    // Two functions are generated:  {fn}_shim and the RouteBuilder returning {fn}

    // A shim function is generated which wraps the user defined function, pulling and parsing
    // the parameters (and eventually any stored state);
    //
    // The route function name is replaced with a function that returns a RouteBuilder containing a reference
    // to the route and the shimmed function.
    let ret = quote_spanned! { input.span() =>
        fn #fn_name<'a>() -> aws_oxide_api::application::RouteBuilder<'a> {
            let route = aws_oxide_api::route::Route::new(
                #method,
                #route,
            ).expect("Major bug in code generation; please report with offending function signature.");

            aws_oxide_api::application::RouteBuilder::new(
                route,
                #fn_shim,
            )
        }

        #(#attrs)*
        async fn #fn_shim(request: aws_oxide_api::OxideRequest, route: aws_oxide_api::application::SharedRoute) -> aws_oxide_api::response::RouteOutcome {
            let #mapping = route.mapped_param_value(request.incoming_route());

            #asyncness fn #fn_actual(#inputs) #ret #body

            #(#param_expansion)*

            // #fn_call
            aws_oxide_api::response::RouteOutcome::Response(
                #fn_actual ( #(#param_ident),* )
                    #await_fn//.await
                    .map(IntoResponse::into_response)
            )
        }
    };

    TokenStream::from(ret)
}


fn parse_route_arguments(args: &AttributeArgs) -> Result<(&LitStr, &LitStr), &str> {
    let method = args
        .get(0)
        .ok_or("Required positional argument 0 as `method`")?;
    let route = args
        .get(1)
        .ok_or("Required positional argument 1 as `route`")?;

    parse_literal_argument(method)
        .and_then(|method| {
            let route = parse_literal_argument(route)?;
            Ok((method, route))
        })
}


fn parse_literal_argument(lit: &NestedMeta) -> Result<&LitStr, &str> {
    match lit {
        NestedMeta::Meta(_) => Err("Argument must be a string literal"),
        NestedMeta::Lit(lit) => {
            match lit {
                Lit::Str(l) => Ok(l),
                _ => Err("Argument must be a string literal")
            }
        }
    }
}


fn parameter_match_expansion(mapping: &Ident, parameter: &Parameter) -> Result<(Ident, impl ToTokens), &'static str> {
    let param_name = parameter.param_name;
    let pname_v = format_ident!("{}_v", param_name);
    let param_type = &parameter.param_type;
    let param_type_lit = format!("{}", param_type.param_type);

    let tokens = match &*param_type_lit {
        "Body" => { // TODO: This may be better suited as a SharedBody, an Arc<&Body>
            quote! {
                let #pname_v = match request.body() {
                    lambda_http::Body::Empty => lambda_http::Body::Empty,
                    lambda_http::Body::Text(body) => lambda_http::Body::Text(body.clone()),
                    lambda_http::Body::Binary(body) => lambda_http::Body::Binary(body.clone()),
                };
            }
        },
        "OxideRequest" => {
            quote! {
                let #pname_v = request.clone();
            }
        },
        "Json" => json_parameter_match(&parameter, &pname_v)?,
        _ => { // default behavior is to try and match as a parameter from the route
            default_parameter_match(&mapping, &parameter, &pname_v)
        }
    };

    Ok((pname_v, tokens))
}


/// Generates code which attempts to parse
fn default_parameter_match(mapping: &Ident, parameter: &Parameter, pname_v: &Ident) -> proc_macro2::TokenStream {
    let param_name = parameter.param_name;
    let ptype_parse = parameter.param_type.param_path;
    let pname_lit = format!("{}", param_name);

    quote! {
        let #pname_v = if let Some(#pname_v) = #mapping.get(#pname_lit) {
            if let Ok(v) = #pname_v.parse::<#ptype_parse>() {
                v
            } else {
                return aws_oxide_api::response::RouteOutcome::Forward;
            }
        } else {
            return aws_oxide_api::response::RouteOutcome::Forward;
        };
    }
}

fn json_parameter_match(parameter: &Parameter, pname_v: &Ident) -> Result<proc_macro2::TokenStream, &'static str> {
    let generic_type = extract_parameter_generic_type(parameter)
        .ok_or("Json requires a supported generic type")?;

    let generic_type_ident = format_ident!("{}", generic_type);
    let pname_v_header = format_ident!("{}_content_type_header", pname_v);
    let pname_v_body = format_ident!("{}_body", pname_v);
    let pname_v_deser = format_ident!("{}_deser", pname_v);

    // TODO: we should re-export lambda_http packages as part of aws_oxide_api
    // to resolve some of these import issues
    let tokens = quote! {
        let #pname_v_header = request
            .headers()
            .get(aws_oxide_api::http::header::CONTENT_TYPE);

        // if the content type doesn't match the expected application/json
        // return
        if let Some(content_type) = #pname_v_header {
            let content_type_str = if let Ok(content_type) = content_type.to_str() {
                content_type
            } else {
                return aws_oxide_api::response::RouteOutcome::Forward;
            };

            if content_type_str.to_lowercase() != "application/json" {
                return aws_oxide_api::response::RouteOutcome::Forward;
            };
        } else {
            return aws_oxide_api::response::RouteOutcome::Forward;
        };

        let #pname_v_body: String = match request.body() {
            aws_oxide_api::lambda_http::Body::Text(body) => body.clone(),
            _ => return aws_oxide_api::response::RouteOutcome::Forward,
        };

        let #pname_v_deser: #generic_type_ident = match serde_json::from_str(&#pname_v_body) {
            Ok(v) => v,
            Err(_) => return aws_oxide_api::response::RouteOutcome::Forward,
        };

        let #pname_v: aws_oxide_api::parameters::Json<#generic_type_ident> = aws_oxide_api::parameters::Json::new(#pname_v_deser);
    };

    Ok(tokens)
}


fn extract_parameter<'a>(pat_type: &'a PatType) -> Option<Parameter> {
    let param_name = extract_parameter_name(pat_type);

    param_name
        .and_then(|param_name| {
            extract_parameter_type(pat_type)
                .and_then(|param_type| {
                    Some(
                        Parameter {
                            param_name,
                            param_type,
                        }
                    )
                })
        })
}


fn extract_parameter_name<'a>(pat_type: &'a PatType) -> Option<&'a Ident> {
    match &*pat_type.pat {
        Pat::Ident(pat_ident) => {
            Some(&pat_ident.ident)
        },
        _ => {
            None
        }
    }
}


fn extract_parameter_type<'a>(pat_type: &'a PatType) -> Option<ParameterType> {
    match &*pat_type.ty {
        Type::Path(type_path) => {
            let segment = type_path.path.segments.first()?;
            let param_type = &segment.ident;
            let param_path = &type_path.path;

            Some(
                ParameterType {
                    param_path,
                    param_type,
                }
            )
        },
        _ => None,
    }
}


fn extract_parameter_generic_type<'a>(parameter: &'a Parameter) -> Option<&'a Ident> {
    let param_path = parameter.param_type.param_path;
    let segment = param_path.segments.first()?;

    match &segment.arguments {
        PathArguments::AngleBracketed(generic) => {
            match generic.args.first()? {
                GenericArgument::Type(generic_type) => {
                    match generic_type {
                        Type::Path(type_path) => {
                            let segment = type_path.path.segments
                                .first()?;

                            Some(&segment.ident)
                        },
                        _ => None,
                    }
                },
                _ => None,
            }
        },
        _ => None
    }
}

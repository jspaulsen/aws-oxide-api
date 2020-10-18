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

            async fn #fn_actual(#inputs) #ret #body

            #(#param_expansion)*

            aws_oxide_api::response::RouteOutcome::Response(
                #fn_actual ( #(#param_ident),* )
                    .await
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
        // TODO: This may be better suited as a SharedBody, an Arc<&Body>
        "Body" => {
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
        _ => { // default behavior is to try and match as a parameter from the route
            parameter_match(&mapping, &parameter, &pname_v)
        }
    };

    Ok((pname_v, tokens))
}


fn parameter_match(mapping: &Ident, parameter: &Parameter, pname_v: &Ident) -> proc_macro2::TokenStream {
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


fn _extract_parameter_generic_type<'a>(parameter: &'a Parameter) -> Option<&'a Ident> {
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

extern crate proc_macro;
use std::str::FromStr;
use aws_oxide_api_route::{
    Route,
    RouteUri,
};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    AttributeArgs,
    Error as SynError,
    FnArg,
    GenericArgument,
    Ident,
    ImplItemMethod,
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
    param_type: &'a Ident,
    param_path: &'a syn::Path,
}


#[derive(Debug)]
struct Parameter<'a> {
    param_name: &'a Ident,
    param_type: ParameterType<'a>,
}



#[proc_macro_attribute]
/// `route`
///
/// The `route` macro transforms a user defined function into a function in which
/// aws-oxide-api can route a request to.
/// # Arguments
/// * `method` - HTTP Method
/// * `route` - Route path to match;
///
/// Dynamic segments in the path can be defined with a prefix of `:`, e.g.,
/// ```ignore
/// #[route("POST", "/some/:id_a/another/:id_b")]
/// ```
/// which can then be defined as arguments.
///
/// # Examples
/// ```ignore
/// #[route("POST", "/some/:id_a/another/:id_b")]
/// async fn example_route(id_a: i32, id_b: String, body: Binary) -> Result<impl IntoResponse, ResponseError> {
///    Ok("")
///}
/// ```
pub fn route(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = syn::parse_macro_input!(item as ImplItemMethod);
    let vis = &input.vis;
    let asyncness = &input.sig.asyncness;
    let attrs = &input.attrs;
    let return_type = &input.sig.output;
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

    let parsed_route = RouteUri::from_str(
        &route.value()
    ).expect("RouteUri failed to parse despite earlier validation");

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
                    &parsed_route,
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

    let result = if is_result_type(return_type) {
        let ret = quote! {
            let result = result
                .map(aws_oxide_api::IntoResponse::into_response);
        };

        Some(ret)
    } else {
        let ret = quote! {
            //let intermediate = let #pname_v: #param_type = match <#param_type as aws_oxide_api::guards::Guard>::from_request(request).await
            let result = Ok(result.into_response());
        };

        Some(ret)
    };


    let ret = quote_spanned! { input.span() =>
        // Take and rename the user provided function
        #asyncness fn #fn_actual(#inputs) #return_type #body

        // Take the existing {fn} and redefine as a function which returns a StoredRoute
        // This is consumed by ApplicationBuilder via `add_route`
        #vis fn #fn_name() -> aws_oxide_api::application::StoredRoute {
            let route = aws_oxide_api::route::Route::new(
                #method,
                #route,
            ).expect("Major bug in code generation; please report with offending function signature.");

            aws_oxide_api::application::StoredRoute {
                route: std::sync::Arc::new(route),
                func: #fn_shim,
            }
        }

        #(#attrs)*
        pub fn #fn_shim<'a>(request: &'a aws_oxide_api::RouteRequest<'_>, route: aws_oxide_api::application::SharedRoute) -> aws_oxide_api::futures::future::BoxFuture<'a, aws_oxide_api::response::RouteOutcome> {
            pub async fn inner_shim(request: &'_ aws_oxide_api::RouteRequest<'_>, route: aws_oxide_api::application::SharedRoute) -> aws_oxide_api::response::RouteOutcome {
                let #mapping = route.mapped_param_value(request.incoming_route());

                #(#param_expansion)*

                let result = #fn_actual ( #(#param_ident),* )
                    #await_fn;

                // generate result based on the result type of the calling
                // function
                #result

                aws_oxide_api::response::RouteOutcome::Response(result)
            };

            Box::pin(inner_shim(request, route))
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


fn parameter_match_expansion(route_template: &RouteUri, mapping: &Ident, parameter: &Parameter) -> Result<(Ident, impl ToTokens), &'static str> {
    let param_name = parameter.param_name;
    let pname_v = format_ident!("{}_v", param_name);
    let param_name_lit = format!("{}", param_name);

    // if the parameter is found in the route template, generate parameter
    // match code
    let tokens = if route_template.contains_parameter(&param_name_lit) {
        parameter_match(&mapping, &parameter, &pname_v)
    } else { /* if the parameter is not found in the template it must be a guard */
        guard_match(&parameter, &pname_v)
    };

    Ok((pname_v, tokens))
}


fn guard_match(parameter: &Parameter, pname_v: &Ident) -> proc_macro2::TokenStream {
    let generic_type = extract_parameter_generic_type(parameter);
    let param_type = &parameter.param_type;
    let param_type_ident = format_ident!("{}", param_type.param_type);

    let param_type = match generic_type {
        Some(generic_type) => {
            quote! {
                #param_type_ident::<#generic_type>
            }
        },
        None => {
            quote! {
                #param_type_ident
            }
        }
    };

    quote! {
        let #pname_v: #param_type = match <#param_type as aws_oxide_api::guards::Guard>::from_request(request).await {
            aws_oxide_api::guards::GuardOutcome::Value(v) => v,
            aws_oxide_api::guards::GuardOutcome::Error(err) => return aws_oxide_api::response::RouteOutcome::Response(Ok(err)),
            aws_oxide_api::guards::GuardOutcome::Forward => return aws_oxide_api::response::RouteOutcome::Forward,
        };
    }
}

/// Generates code which attempts to parse
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
        Type::Path(type_path) => extract_type(type_path),
        _ => None,
    }
}


fn extract_type<'a>(type_path: &'a syn::TypePath) -> Option<ParameterType> {
    let segment = type_path
        .path
        .segments
        .first()?;

    Some(
        ParameterType {
            param_path: &type_path.path,
            param_type:  &segment.ident,
        }
    )
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


fn is_result_type(return_type: &syn::ReturnType) -> bool {
    if let syn::ReturnType::Type(_, ty) = return_type {
        match ty.as_ref() {
            Type::Path(type_path) => {
                let parameter_type = extract_type(type_path);

                if let Some(parameter_type) = parameter_type {
                    if parameter_type.param_type == "Result" {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            _ => false,
        }
    } else {
        false
    }
}

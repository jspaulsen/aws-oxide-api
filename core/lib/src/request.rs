use std::{
    collections::HashMap,
    sync::Arc,
};

use aws_oxide_api_route::IncomingRoute;
use state::Container;
use crate::{
    http::{
        HeaderMap,
        Uri,
    },
    LambdaRequest,
    netlify_lambda_http::Body,
};

pub struct InnerRequest {
    request: LambdaRequest,
    incoming: IncomingRoute,
    parameters: HashMap<String, String>,
    pub container: Arc<Container>,
}

#[derive(Clone)]
pub struct OxideRequest {
    pub inner: Arc<InnerRequest>,
}

impl InnerRequest {
    fn new(request: LambdaRequest, container: Arc<Container>) -> Self {
        let incoming = IncomingRoute::from(&request);
        let parameters = parse_query(&request.uri());

        Self {
            request,
            incoming,
            parameters,
            container,
        }
    }
}

impl OxideRequest {
    pub fn new(request: LambdaRequest, container: Arc<Container>) -> Self {
        Self {
            inner: Arc::new(
                InnerRequest::new(
                    request,
                    container,
                )
            )
        }
    }

    pub fn incoming_route(&self) -> &IncomingRoute {
        &self.inner.incoming
    }

    pub fn parameters(&self) -> &HashMap<String, String> {
        &self.inner.parameters
    }

    pub fn body(&self) -> &Body {
        self.inner.request.body()
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.inner.request.headers()
    }
}


pub fn parse_query(uri: &Uri) -> HashMap<String, String> {
    match uri.query() {
        Some(params) => {
            let mut ret: HashMap<String, String> = HashMap::new();
            let split_params: Vec<&str> = params
                .split('&')
                .collect();

            for param_str in &split_params {
                let parsed: Vec<&str> = param_str
                    .split('=')
                    .collect();

                let param = parsed.get(0);
                let value = parsed
                    .get(1)
                    .unwrap_or(&"");

                if let Some(param) = param {
                    ret.insert(
                        param.to_string(),
                        value.to_string(),
                    );
                }
            }

            ret
        },
        None => HashMap::new(),
    }
}


#[cfg(test)]
mod tests {
    use crate::{
        http::Uri,
        request::parse_query,
    };

    #[test]
    fn test_parse_query() {
        let uri = "/foo/bar?abcd=123&defg=456&key#flip".parse::<Uri>()
            .unwrap();

        let parameters = parse_query(&uri);
        let empty = parameters
            .get("key")
            .unwrap();


        assert_eq!(empty, "");
        assert!(parameters.get("abcd").is_some());
        assert_eq!(parameters.get("defg").unwrap(), "456");
        assert!(parameters.get("flip").is_none());
    }

}

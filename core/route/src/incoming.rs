use lambda_http::http::{
    method::Method,
    request::Request,
    uri::Uri,
};

#[derive(Debug)]
pub struct IncomingRouteUri {
    pub segments: Vec<String>,
}

#[derive(Debug)]
pub struct IncomingRoute {
    pub method: Method,
    pub uri: IncomingRouteUri,
}

impl IncomingRoute {
    pub fn get(&self, index: usize) -> Option<&String> {
        self.uri.segments.get(index)
    }
}

impl From<&Uri> for IncomingRouteUri {
    fn from(uri: &Uri) -> Self {
        let segments: Vec<String> = uri
            .path()
            .split('/')
            .into_iter()
            .filter(|e| !e.is_empty())
            .map(|e| e.into())
            .collect();

        Self {
            segments
        }
    }
}

impl<T> From<&Request<T>> for IncomingRoute {
    fn from(request: &Request<T>) -> Self {
        Self {
            uri: IncomingRouteUri::from(request.uri()),
            method: request
                .method()
                .clone()
        }
    }
}

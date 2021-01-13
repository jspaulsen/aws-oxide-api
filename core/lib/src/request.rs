use std::sync::Arc;

use aws_oxide_api_route::IncomingRoute;
use state::Container;
use crate::{
    Body,
    http::HeaderMap,
    LambdaRequest,
    netlify_lambda_http::{
        RequestExt,
        StrMap,
    },
};

pub struct InnerRequest {
    request: LambdaRequest,
    incoming: IncomingRoute,
}

#[derive(Clone)]
pub struct OxideRequest {
    pub inner: Arc<InnerRequest>,
    pub container: Arc<Container>,
}

impl InnerRequest {
    fn new(request: LambdaRequest) -> Self {
        let incoming = IncomingRoute::from(&request);

        Self {
            request,
            incoming,
        }
    }
}

impl OxideRequest {
    pub fn new(request: LambdaRequest, container: Arc<Container>) -> Self {
        Self {
            inner: Arc::new(InnerRequest::new(request)),
            container,
        }
    }

    pub fn incoming_route(&self) -> &IncomingRoute {
        &self.inner.incoming
    }

    pub fn parameters(&self) -> StrMap {
        self
            .inner
            .request
            .query_string_parameters()
    }

    pub fn body(&self) -> &Body {
        self.inner.request.body()
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.inner.request.headers()
    }
}

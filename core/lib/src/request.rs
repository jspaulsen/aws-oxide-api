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

pub struct RouteRequest<'a> {
    pub inner: Arc<InnerRequest>,
    pub container: &'a Container,
}

/// Represents an incoming request
pub struct Request {
    inner: Arc<InnerRequest>
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

impl<'a> RouteRequest<'a> {
    pub fn new(request: LambdaRequest, container: &'a Container) -> Self {
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
        self
            .inner
            .request
            .body()
    }

    pub fn headers(&self) -> &HeaderMap {
        &self
            .inner
            .request
            .headers()
    }

    /// Converts an internal request into one which is
    /// exposed to a route function
    pub fn as_request(&self) -> Request {
        Request::new(
            self
                .inner
                .clone()
        )
    }
}

impl Request {
    pub fn new(request: Arc<InnerRequest>) -> Self {
        Self {
            inner: request
        }
    }

    pub fn incoming_route(&self) -> &IncomingRoute {
        &self
            .inner
            .incoming
    }

    pub fn parameters(&self) -> StrMap {
        self
            .inner
            .request
            .query_string_parameters()
    }

    pub fn body(&self) -> &Body {
        self
            .inner
            .request
            .body()
    }

    pub fn headers(&self) -> &HeaderMap {
        &self
            .inner
            .request
            .headers()
    }
}

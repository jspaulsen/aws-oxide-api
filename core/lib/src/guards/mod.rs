use async_trait::async_trait;
use crate::{
    Body,
    LambdaResponse,
    request::{
        RouteRequest,
        Request,
    },
};

pub use json::Json;

mod json;


pub enum GuardOutcome<V> {
    Value(V),
    Error(LambdaResponse),
    Forward,
}

#[async_trait]
pub trait Guard<'a, 'r>: Sized {
    async fn from_request(request: &'a RouteRequest<'r>) -> GuardOutcome<Self>;
}

#[async_trait]
impl<'a, 'r> Guard<'a, 'r> for Request {
    async fn from_request(request: &'a RouteRequest<'r>) -> GuardOutcome<Self> {
        GuardOutcome::Value(
            request
                .as_request()
        )
    }
}

#[async_trait]
impl<'a, 'r> Guard<'a, 'r> for Body {
    async fn from_request(request: &'a RouteRequest<'r>) -> GuardOutcome<Self> {
        let body = match request.body() {
            Body::Empty => Body::Empty,
            Body::Text(body) => Body::Text(body.clone()),
            Body::Binary(body) => Body::Binary(body.clone()),
        };

        GuardOutcome::Value(body)
    }
}

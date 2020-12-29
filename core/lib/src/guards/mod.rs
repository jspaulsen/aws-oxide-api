use async_trait::async_trait;
use crate::{
    Body,
    IntoResponse,
    LambdaResponse,
    request::OxideRequest,
};

pub use json::Json;

mod json;

pub type BoxedIntoResponse = Box<dyn IntoResponse>;

pub enum GuardOutcome<V> {
    Value(V),
    Error(LambdaResponse),
    Forward,
}

#[async_trait]
pub trait Guard: Sized {
    async fn from_request(request: OxideRequest) -> GuardOutcome<Self>;
}

#[async_trait]
impl Guard for OxideRequest {
    async fn from_request(request: OxideRequest) -> GuardOutcome<Self> {
        GuardOutcome::Value(request)
    }
}

#[async_trait]
impl Guard for Body {
    async fn from_request(request: OxideRequest) -> GuardOutcome<Self> {
        let body = match request.body() {
            Body::Empty => Body::Empty,
            Body::Text(body) => Body::Text(body.clone()),
            Body::Binary(body) => Body::Binary(body.clone()),
        };

        GuardOutcome::Value(body)
    }
}

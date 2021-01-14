use async_trait::async_trait;
use crate::{
    LambdaResponse,
    request::{
        RouteRequest,
        Request,
    },
};

pub use body::{
    Binary,
    Text,
};
pub use json::Json;

mod body;
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

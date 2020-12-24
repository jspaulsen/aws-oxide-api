use crate::{
    IntoResponse,
    LambdaResponse,
    request::OxideRequest,
};

pub use body::RequestBody;
pub use json::Json;

mod body;
mod json;

pub type BoxedIntoResponse = Box<dyn IntoResponse>;

pub enum GuardOutcome<V> {
    Value(V),
    Error(LambdaResponse),
    Forward,
}

pub trait Guard: Sized {
    fn from_request(request: OxideRequest) -> GuardOutcome<Self>;
}

impl Guard for OxideRequest {
    fn from_request(request: OxideRequest) -> GuardOutcome<Self> {
        GuardOutcome::Value(request)
    }
}

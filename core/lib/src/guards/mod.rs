use crate::{
    request::OxideRequest,

};

pub use body::RequestBody;
pub use json::Json;

mod body;
mod json;


pub trait Guard: Sized {
    fn from_request(request: OxideRequest) -> Option<Self>;
}

impl Guard for OxideRequest {
    fn from_request(request: OxideRequest) -> Option<Self> {
        Some(request)
    }
}

use std::error::Error;
use std::fmt;

use lambda_http::http as http;

#[derive(Debug)]
pub enum RouteError {
    InvalidMethod,
    InvalidRoute { route: String },
    InvalidSegmentParameter { segment: String },

}

impl fmt::Display for RouteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMethod => write!(f, "InvalidMethod"),
            Self::InvalidRoute { route } => write!(f, "InvalidRoute `{}`", route),
            Self::InvalidSegmentParameter { segment } => write!(f, "InvalidSegmentParameter `{}`", segment),
        }
    }
}

impl RouteError {
    pub fn invalid_uri(uri: &str) -> Self {
        Self::InvalidRoute {
            route: uri.into()
        }
    }
}

impl Error for RouteError {}


impl From<http::method::InvalidMethod> for RouteError {
    fn from(_: http::method::InvalidMethod) -> Self {
        Self::InvalidMethod
    }
}

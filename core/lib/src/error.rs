use std::error::Error;
use std::fmt;

use aws_oxide_api_route::error::RouteError;

#[derive(Debug)]
pub enum OxideError {
    RouteError(RouteError),
}

impl fmt::Display for OxideError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RouteError(e) => e.fmt(f),
        }
    }
}

impl Error for OxideError {}

impl From<RouteError> for OxideError {
    fn from(e: RouteError) -> Self {
        Self::RouteError(e)
    }
}

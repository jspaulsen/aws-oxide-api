use std::{
    ops::{
        Deref,
        DerefMut,
    },
};
use serde::de::DeserializeOwned;
use serde_json;
use crate::{
    guards::Guard,
    http,
    lambda_http::Body,
    request::OxideRequest,
};


pub struct Json<T: DeserializeOwned>(T);

impl<T: DeserializeOwned> Json<T> {
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn new(t: T) -> Self {
        Self(t)
    }
}

impl<T: DeserializeOwned> Deref for Json<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: DeserializeOwned> DerefMut for Json<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: DeserializeOwned> Guard for Json<T> {
    fn from_request(request: OxideRequest) -> Option<Self> {
        let header = request
            .headers()
            .get(http::header::CONTENT_TYPE);

        // Ensure Content-Type is application/json
        if let Some(content_type) = header {
            if let Ok(content_type) = content_type.to_str() {
                if content_type.to_lowercase() != "application/json" {
                    return None;
                }
            } else {
                return None;
            };
        } else {
            return None;
        };

        match request.body() {
            Body::Text(body) => {
                let deser: Result<T, _> = serde_json::from_str(body);

                if let Ok(deser) = deser {
                    Some(Self(deser))
                } else {
                    None
                }
            },
            _ => None
        }
    }
}

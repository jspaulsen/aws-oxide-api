use std::{
    ops::{
        Deref,
        DerefMut,
    },
};
use crate::{
    guards::Guard,
    lambda_http::Body,
    request::OxideRequest,
};


pub struct RequestBody(Body);

impl RequestBody {
    #[inline(always)]
    pub fn into_inner(self) -> Body {
        self.0
    }
}

impl Deref for RequestBody {
    type Target = Body;

    #[inline(always)]
    fn deref(&self) -> &Body {
        &self.0
    }
}

impl DerefMut for RequestBody {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Body {
        &mut self.0
    }
}

impl Guard for RequestBody {
    fn from_request(request: OxideRequest) -> Option<Self> {
        let body = match request.body() {
            Body::Empty => Body::Empty,
            Body::Text(body) => Body::Text(body.clone()),
            Body::Binary(body) => Body::Binary(body.clone()),
        };

        Some(RequestBody(body))
    //         quote! {
    //             let #pname_v = match request.body() {
    //                 lambda_http::Body::Empty => lambda_http::Body::Empty,
    //                 lambda_http::Body::Text(body) => lambda_http::Body::Text(body.clone()),
    //                 lambda_http::Body::Binary(body) => lambda_http::Body::Binary(body.clone()),
    //             };
    //         }
    }
}

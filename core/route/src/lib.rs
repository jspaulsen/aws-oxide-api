use std::{
    collections::HashMap,
    str::FromStr,
};

use netlify_lambda_http::http as http;

use crate::{
    route_uri::{
        RouteSegment,
    },
};

pub use error::RouteError;
pub use incoming::IncomingRoute;
pub use route_uri::RouteUri;

pub mod error;
mod incoming;
mod route_uri;


#[derive(Debug)]
pub struct Route {
    uri: RouteUri,
    method: http::Method,
}

impl Route {
    pub fn validate<R: AsRef<str>>(method: R, uri: R) -> Result<(), RouteError> {
        RouteUri::from_str(
            uri.as_ref()
        ).and(
            http::Method::from_str(
                method.as_ref()
            )
            .map_err(|e| e.into())
        ).map(|_| ())
    }

    pub fn new<R: AsRef<str>>(method: R, uri: R) -> Result<Self, RouteError> {
        let ret = Self {
            uri: RouteUri::from_str(uri.as_ref())?,
            method: http::Method::from_str(method.as_ref())?,
        };

        Ok(ret)
    }

    pub fn matches(&self, incoming: &IncomingRoute) -> bool {
        if self.method == incoming.method {
            if self.uri.matches(&incoming.uri) {
                return true;
            }
        }

        false
    }

    pub fn mapped_param_value<'a>(&'_ self, incoming: &'a IncomingRoute) -> HashMap<&'_ str, &'a str> {
        let mut ret = HashMap::new();
        let enumerate = self.uri
            .segments()
            .iter()
            .enumerate();

        for (index, segment) in enumerate {
            match segment {
                RouteSegment::Dynamic { parameter } => {
                    let incoming_segment = incoming.get(index);

                    match incoming_segment {
                        Some(value) => {
                            ret.insert(
                                parameter.as_ref(),
                                value.as_ref(),
                            );
                        },
                        None => continue,
                    }
                },
                RouteSegment::Constant(..) => continue,
            }
        }

        ret
    }


}


#[cfg(test)]
mod tests {
    use netlify_lambda_http::{
        Body,
        http::Request,
    };

    use crate::{
        IncomingRoute,
        Route,
    };

    #[test]
    fn test_matches_const() {
        let incoming_request = Request::builder()
            .method("POST")
            .uri("/foo/bar/baz")
            .body(Body::Empty)
            .unwrap();
        let incoming_route = IncomingRoute::from(&incoming_request);

        let matching_route = Route::new(
            "POST",
            "/foo/bar/baz",
        ).unwrap();

        let nonmatch_method = Route::new(
            "GET",
            "/foo/bar/baz"
        ).unwrap();

        let nonmatch_length = Route::new(
            "POST",
            "/foo/bar"
        ).unwrap();

        assert!(matching_route.matches(&incoming_route));
        assert!(!nonmatch_method.matches(&incoming_route));
        assert!(!nonmatch_length.matches(&incoming_route));
    }

    #[test]
    fn test_match_root() {
        let incoming_request = Request::builder()
            .method("POST")
            .uri("/")
            .body(Body::Empty)
            .unwrap();
        let incoming_route = IncomingRoute::from(&incoming_request);

        let matching_route = Route::new(
            "POST",
            "/",
        ).unwrap();

        let nonmatch_method = Route::new(
            "GET",
            "/",
        ).unwrap();

        let nonmatch_length = Route::new(
            "POST",
            "/foo",
        ).unwrap();


        assert!(matching_route.matches(&incoming_route));
        assert!(!nonmatch_length.matches(&incoming_route));
        assert!(!nonmatch_method.matches(&incoming_route));
    }

    #[test]
    fn test_match_dynamic() {
        let incoming_request = Request::builder()
            .method("POST")
            .uri("/foo/abcd/baz")
            .body(Body::Empty)
            .unwrap();
        let incoming_route = IncomingRoute::from(&incoming_request);

        let matching_route = Route::new(
            "POST",
            "/foo/:id/baz",
        ).unwrap();

        let nonmatch_route_method = Route::new(
            "GET",
            "/foo/:id/baz",
        ).unwrap();

        let nonmatch_const = Route::new(
            "POST",
            "/:id/bar/baz",
        ).unwrap();

        assert!(matching_route.matches(&incoming_route));
        assert!(!nonmatch_route_method.matches(&incoming_route));
        assert!(!nonmatch_const.matches(&incoming_route));
    }
}

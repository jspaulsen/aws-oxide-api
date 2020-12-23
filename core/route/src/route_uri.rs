use std::{
    convert::TryFrom,
    str::FromStr,
};

use lambda_http::http as http;

use crate::{
    error::RouteError,
    incoming::IncomingRouteUri,
};


#[derive(Debug, PartialEq)]
pub enum RouteSegment {
    Constant(String),
    Dynamic { parameter: String },
}

#[derive(Debug)]
pub struct RouteUri {
    // if empty, considered to be a root route, i.e., `/`
    segments: Vec<RouteSegment>,
}

impl RouteUri {
    pub fn matches(&self, incoming: &IncomingRouteUri) -> bool {
        let enumerate = self.segments
            .iter()
            .enumerate();

        if self.segments.len() != incoming.segments.len() {
            return false;
        }

        for (i, route_segment) in enumerate {
            let incoming_segment = &incoming.segments[i];

            match route_segment {
                RouteSegment::Constant(segment) => {
                    if segment != incoming_segment {
                        return false;
                    }
                },
                RouteSegment::Dynamic {..} => {
                    continue;
                }
            }
        }

        true
    }

    pub fn segments(&self) -> &Vec<RouteSegment> {
        &self.segments
    }

    /// Checks if the RouteUri has a dynamic segment with a
    /// given named parameter
    ///
    /// # Arguments
    /// * `param` - Parameter name to lookup
    pub fn contains_parameter(&self, param: &str) -> bool {
        for segment in &self.segments {
            if let RouteSegment::Dynamic { parameter } = segment {
                if parameter == param {
                    return true;
                }
            }
        }

        false
    }
}

impl FromStr for RouteUri {
    type Err = RouteError;

    fn from_str(route: &str) -> Result<Self, Self::Err> {
        let parsed_uri = http::Uri::try_from(route)
            .map_err(|_| Self::Err::invalid_uri(route))?;
        let mut segmented_route = parsed_uri.path();

        if !route.is_empty() {
            if segmented_route.starts_with("/") {
                segmented_route = &segmented_route[1..];
            }

            if segmented_route.ends_with("/") {
                segmented_route = &segmented_route[..segmented_route.len() - 1];
            }
        }

        let segments = segmented_route
            .split("/")
            .into_iter()
            .filter(|e| !e.is_empty())
            .map(|e| e.parse::<RouteSegment>())
            .collect::<Result<Vec<RouteSegment>, RouteError>>()?;

        Ok(Self { segments })
    }
}

impl FromStr for RouteSegment {
    type Err = RouteError;

    fn from_str(segment: &str) -> Result<Self, Self::Err> {
        if segment.starts_with(':') {
            if segment.len() == 1 {
                Err(RouteError::InvalidSegmentParameter { segment: segment.into() })
            } else {
                Ok(Self::Dynamic { parameter: segment[1..].into() })
            }
        } else {
            Ok(Self::Constant(segment.into()))
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::{
        RouteUri,
        route_uri::RouteSegment,
    };


    #[test]
    fn test_parse_constants() {
        let root = "/".parse::<RouteUri>().unwrap();

        // TODO: what should this behavior be?
        let _double_slash = "//".parse::<RouteUri>().unwrap();
        let consts = "/foo/bar/baz/".parse::<RouteUri>().unwrap();

        assert_eq!(root.segments, vec![]);
        assert_eq!(
            consts.segments,
            vec![
                RouteSegment::Constant("foo".into()),
                RouteSegment::Constant("bar".into()),
                RouteSegment::Constant("baz".into()),
            ],
        );
    }

    #[test]
    fn test_parse_dynamic() {
        let dynamic = "/foo/:id_a/:id_b/baz/:id_c/".parse::<RouteUri>().unwrap();

        assert_eq!(
            dynamic.segments,
            vec![
                RouteSegment::Constant("foo".into()),
                RouteSegment::Dynamic { parameter: "id_a".into() },
                RouteSegment::Dynamic { parameter: "id_b".into() },
                RouteSegment::Constant("baz".into()),
                RouteSegment::Dynamic { parameter: "id_c".into() },
            ],
        );
    }

    #[test]
    fn test_parse_errors() {
        let err_empty = "".parse::<RouteUri>();
        let err_character = "/foo/{}/bar/baz".parse::<RouteUri>();
        let err_parameter = "/:foo/:/".parse::<RouteUri>();
        let err_no_starting_slash = "foo/bar/baz".parse::<RouteUri>();

        assert!(err_empty.is_err());
        assert!(err_character.is_err());
        assert!(err_parameter.is_err());
        assert!(err_no_starting_slash.is_err());
    }

    #[test]
    fn test_contains_parameter() {
        let dynamic = "/foo/:id_a/:id_b/baz/:id_c/".parse::<RouteUri>().unwrap();

        assert_eq!(dynamic.contains_parameter("id_a"), true);
        assert_eq!(dynamic.contains_parameter("id_b"), true);
        assert_eq!(dynamic.contains_parameter("id_c"), true);
        assert_eq!(dynamic.contains_parameter("foo"), false);
    }
}

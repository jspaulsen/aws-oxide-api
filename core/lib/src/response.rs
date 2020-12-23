use crate::{
    http,
    lambda_http::{
        Body,
        IntoResponse,
        Response,
    },
    outcome::Outcome,
};

use serde_json::{
    json,
    Value,
};


pub type ResponseError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type LambdaResponse = Response<Body>;
pub type ResponseResult = Result<LambdaResponse, ResponseError>;
pub type RouteOutcome = Outcome<ResponseResult>;

/// Represents a JSON response, returning a JSON payload with a
/// given status code and relevant Content-Type header
pub struct JsonResponse {
    value: Value,
    status_code: u16,
}


impl JsonResponse {
    /// Return a JsonResponse with the given JSON payload and status code
    ///
    /// # Arguments
    /// * `value` - JSON payload
    /// * `status_code` - Response status_code
    pub fn new(value: Value, status_code: u16) -> JsonResponse {
        Self {
            value,
            status_code,
        }
    }

    /// Returns a JsonResponse with a status_code of `400`
    ///
    /// # Arguments
    /// * `value` - Optional payload; by default will return `{"message": "Bad Request"}`
    pub fn bad_request(value: Option<Value>) -> Self {
        let body = value
            .unwrap_or(json!({"message": "Bad Request"}));

        JsonResponse::new(body, 400)
    }
}

impl IntoResponse for JsonResponse {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .header(http::header::CONTENT_TYPE, "application/json")
            .status(self.status_code)
            .body(
                self.value
                    .to_string()
                    .into()
            )
            .expect("Failed to transform JsonResponse into Response")
    }
}

pub fn method_not_found() -> LambdaResponse {
    let mut ret = LambdaResponse::new(Body::Empty);

    *ret.status_mut() = http::StatusCode::METHOD_NOT_ALLOWED;
    ret
}

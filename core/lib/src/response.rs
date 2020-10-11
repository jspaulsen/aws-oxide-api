use lambda_http::{
    Body as LambdaHttpBody,
    http::StatusCode,
    Response,
};

use crate::{
    outcome::Outcome,
};


pub type ResponseError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type LambdaResponse = Response<LambdaHttpBody>;
pub type ResponseResult = Result<LambdaResponse, ResponseError>;
pub type RouteOutcome = Outcome<ResponseResult>;


pub fn method_not_found() -> LambdaResponse {
    let mut ret = LambdaResponse::new(LambdaHttpBody::Empty);

    *ret.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
    ret
}

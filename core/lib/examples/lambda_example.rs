use aws_oxide_api::{
    Application,
    IntoResponse,
    ResponseError,
    route,
};

use lambda_http::{
    Body,
    Context,
    lambda::lambda,
    Request as LambdaRequest,
};

use serde_json::json;


// not a runnable example as it is expected
// to be called from the lambda runtime
#[lambda(http)]
#[tokio::main]
async fn main(request: LambdaRequest, context: Context) -> Result<impl IntoResponse, ResponseError> {
    let mut app = Application::builder()
        .add_route(example_id)
        .build()
        .expect("Failed to build application!");

    app
        .handle(request, context)
        .await
}


#[route("GET", "/example/:id")]
async fn example_id(id: i32, _body: Body) -> Result<impl IntoResponse, ResponseError> {
    Ok(json!({"id": id}))
}

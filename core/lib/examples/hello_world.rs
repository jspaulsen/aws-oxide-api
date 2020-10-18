use aws_oxide_api::{
    Application,
    IntoResponse,
    ResponseError,
    route,
};

use lambda_http::{
    Body,
    http as http,
    lambda::{Context},
};

use serde_json::json;

//#[lambda(http)]
#[tokio::main]
async fn main(/* request: Request, context: Context */) {
    let mut app = Application::builder()
        .add_route(hello)
        .build()
        .unwrap();

    let request = http::Request::builder()
        .method("GET")
        .uri("/some/12345")
        .body(Body::Empty)
        .unwrap();

    let result = app
        .handle(request, Context::default())
        .await
        .unwrap()
        .into_response();

    assert_eq!(result.status().as_u16(), 200);
}


#[route("GET", "/some/:id")]
async fn hello(id: i32) -> Result<impl IntoResponse, ResponseError> {
    Ok(json!({"id": id}))
}

use aws_oxide_api::{
    Application,
    Body,
    Context,
    http,
    IntoResponse,
    route,
};

use serde_json::json;

//#[lambda(http)]
#[tokio::main]
async fn main(/* request: Request, context: Context */) {
    let mut app = Application::builder()
        .add_route(hello)
        .add_route(sync_example)
        .build()
        .unwrap();

    let request = http::Request::builder()
        .method("GET")
        .uri("/some/12345")
        .body(Body::Empty)
        .unwrap();

    let req_sync = http::Request::builder()
        .method("POST")
        .uri("/some/12345")
        .body(Body::Empty)
        .unwrap();

    let result = app
        .handle(request, Context::default())
        .await
        .unwrap()
        .into_response();

    let result_sync = app
        .handle(req_sync, Context::default())
        .await
        .unwrap()
        .into_response();

    assert_eq!(result.status().as_u16(), 200);
    assert_eq!(result_sync.status().as_u16(), 200);
}


#[route("GET", "/some/:id")]
async fn hello(id: i32) -> impl IntoResponse {
    json!({"id": id})
}


#[route("POST", "/some/:id")]
fn sync_example(id: i32) -> serde_json::Value {
    json!({"id": id})
}

use aws_oxide_api::{
    Application,
    IntoResponse,
    ResponseError,
    Request,
    route,
    TestApplication,
};

use lambda_http::{
    Body,
    Response,
};

use serde_json::{
    json,
};


// Normally this would live in a test block; this is an example
// of how the test application can be used.
#[tokio::main]
async fn main() {
    let mut app = TestApplication::new(
        Application::builder()
            .add_route(example)
            .build()
            .unwrap()
    );
    let expected_id: i32 = 12345;
    let uri = format!("/some/{}", expected_id);

    let result = app
        .get(uri, None)
        .await
        .unwrap();

    assert_eq!(result.status().as_u16(), 202);

    match result.body() {
        Body::Text(json) => {
            let body: serde_json::Value = serde_json::from_str(json)
                .unwrap();

            let id = body
                .get("id")
                .unwrap();

            assert_eq!(&json!(expected_id), id);

        },
        _ => unreachable!(),
    };
}



#[route("GET", "/some/:id")]
async fn example(id: i32) -> Result<impl IntoResponse, ResponseError> {
    let response = Response::builder()
        .header("Content-Type", "application/json")
        .status(202) // for examples sake
        .body(
            Body::Text(json!({"id": id}).to_string())
         )?;

    Ok(response)
}

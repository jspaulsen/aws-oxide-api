use aws_oxide_api::{
    Application,
    IntoResponse,
    ResponseError,
    Request,
    route,
};

use lambda_http::{
    Body,
    Context,
    http as http,
    lambda::lambda,
    Request,
};


#[lambda(http)]
#[tokio::main]
async fn main(request: Request, context: Context) -> Result<impl IntoResponse, ResponseError> {
    let mut app = Application::builder()
        .add_route(hello)
        .add_route(there)
        .build()
        .expect("Failed to build application!");

    app
        .handle(request, context)
        .await
}


#[route("GET", "/:id")]
async fn hello(id: i32) -> Result<impl IntoResponse, ResponseError> {
    println!("id is {}", id);

    Ok(())
}

#[route("GET", "/:id")]
async fn there(id: String, body: Body, state: Request) -> Result<impl IntoResponse, ResponseError> {
    println!("id is {}", id);
    println!("State is {:?}", state);
    Ok(())
}

use std::mem;

use lambda_http::{
    Body,
    Context,
    http::{
        HeaderMap,
        self,
    },
    IntoResponse,
    Request,
};

use crate::{
    Application,
    response::{
        LambdaResponse,
        ResponseError,
    },
};


pub struct TestApplication<'a>(Application<'a>);


impl<'a> TestApplication<'a> {
    pub fn new(app: Application<'a>) -> Self {
        Self(app)
    }

    pub async fn call(&mut self, request: Request) -> Result<LambdaResponse, ResponseError> {
        self.0
            .handle(
                request,
                Context::default(),
            )
            .await
            .map(IntoResponse::into_response)
    }

    pub async fn get<R: AsRef<str>>(&mut self, uri: R, headers: Option<HeaderMap>) -> Result<LambdaResponse, ResponseError> {
        let mut request = http::Request::builder()
            .method("GET")
            .uri(uri.as_ref())
            .body(Body::Empty)?;

        if let Some(headers) = headers {
            mem::replace(request.headers_mut(), headers);
        }

        self
            .call(request)
            .await
    }

    pub async fn post<R: AsRef<str>>(&mut self, uri: R, headers: Option<HeaderMap>, body: Body) -> Result<LambdaResponse, ResponseError> {
        let mut request = http::Request::builder()
            .method("POST")
            .uri(uri.as_ref())
            .body(body)?;

        if let Some(headers) = headers {
            mem::replace(request.headers_mut(), headers);
        }

        self
            .call(request)
            .await
    }

    pub async fn put<R: AsRef<str>>(&mut self, uri: R, headers: Option<HeaderMap>, body: Body) -> Result<LambdaResponse, ResponseError> {
        let mut request = http::Request::builder()
            .method("PUT")
            .uri(uri.as_ref())
            .body(body)?;

        if let Some(headers) = headers {
            mem::replace(request.headers_mut(), headers);
        }

        self
            .call(request)
            .await
    }

    pub async fn patch<R: AsRef<str>>(&mut self, uri: R, headers: Option<HeaderMap>, body: Body) -> Result<LambdaResponse, ResponseError> {
        let mut request = http::Request::builder()
            .method("PATCH")
            .uri(uri.as_ref())
            .body(body)?;

        if let Some(headers) = headers {
            mem::replace(request.headers_mut(), headers);
        }

        self
            .call(request)
            .await
    }

    pub async fn delete<R: AsRef<str>>(&mut self, uri: R, headers: Option<HeaderMap>) -> Result<LambdaResponse, ResponseError> {
        let mut request = http::Request::builder()
            .method("DELETE")
            .uri(uri.as_ref())
            .body(Body::Empty)?;

        if let Some(headers) = headers {
            mem::replace(request.headers_mut(), headers);
        }

        self
            .call(request)
            .await
    }
}


// app
// .handle(request, Context::default())
// .await
// .unwrap();
    // /// Entrypoint from Lambda main
    // pub async fn handle(&mut self, event: LambdaRequest, _context: Context) -> Result<impl IntoResponse, ResponseError> {
    //     self.call_req(
    //         event.into()
    //     ).await
    // }

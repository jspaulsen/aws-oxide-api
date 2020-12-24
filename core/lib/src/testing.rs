use crate::{
    Application,
    Context,
    http::{
        HeaderMap,
        self,
    },
    IntoResponse,
    netlify_lambda_http::{
        Body,
        Request,
    },
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
            request
                .headers_mut()
                .extend(headers);
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
            request
                .headers_mut()
                .extend(headers);
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
            request
                .headers_mut()
                .extend(headers);
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
            request
                .headers_mut()
                .extend(headers);
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
            request
                .headers_mut()
                .extend(headers);
        }

        self
            .call(request)
            .await
    }
}

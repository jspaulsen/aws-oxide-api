use std::{
    sync::Arc,
};

use futures::{
    Future,
};
use lambda_http::{
    Context,
    IntoResponse,
    Request as LambdaRequest,
};

use crate::{
    error::OxideError,
    outcome::Outcome,
    response::{
        method_not_found,
        ResponseError,
        ResponseResult,
        RouteOutcome,
    },
    request::Request,
};

use aws_oxide_api_route::Route;

pub type SharedRoute = Arc<Route>;
type RouteFunction<'a> = Box<dyn FnMut(Request, Arc<Route>) -> futures::future::BoxFuture<'static, RouteOutcome> + Send + 'a>;


/// Route object which is stored and
/// associated with a handler function
struct StoredRoute<'a> {
    route: SharedRoute,
    func: RouteFunction<'a>,
}

pub struct ApplicationBuilder<'a> {
    routes: Vec<StoredRoute<'a>>
}

pub struct Application<'a> {
    routes: Vec<StoredRoute<'a>>,
}

pub struct RouteBuilder<'a> {
    route: Route,
    func: RouteFunction<'a>,
}

impl<'a> ApplicationBuilder<'a> {
    pub fn add_route<F>(mut self, func: F) -> Self
    where
        F: (FnOnce() -> RouteBuilder<'a>) + Send + 'static,
    {
        self.routes
            .push(func().into());

        self
    }

    pub fn build(self) -> Result<Application<'a>, OxideError> {
        let ret = Application::new(
            self.routes
        );

        Ok(ret)
    }
}

impl<'a> RouteBuilder<'a> {
    pub fn new<Func, Fut>(route: Route, mut func: Func) -> Self
    where
        Func: (FnMut(Request, SharedRoute) -> Fut) + Send + 'static,
        Fut: Future<Output = RouteOutcome> + Send + 'static,
    {
        Self {
            route,
            func: Box::new(move |req: Request, route: SharedRoute| Box::pin(func(req, route))),
        }
    }
}

impl<'a> Into<StoredRoute<'a>> for RouteBuilder<'a> {
    fn into(self) -> StoredRoute<'a> {
        StoredRoute {
            route: Arc::new(self.route),
            func: self.func,
        }
    }
}

impl<'a> Default for ApplicationBuilder<'a> {
    fn default() -> Self {
        Self {
            routes: Vec::new(),
        }
    }
}


impl<'a> Application<'a> {
    fn new(routes: Vec<StoredRoute<'a>>) -> Self {
        Self {
            routes,
        }
    }

    pub fn builder() -> ApplicationBuilder<'a> {
        ApplicationBuilder::default()
    }

    /// Entrypoint from Lambda main
    pub async fn handle(&mut self, event: LambdaRequest, _context: Context) -> Result<impl IntoResponse, ResponseError> {
        self.call_req(
            event.into()
        ).await
    }

    /// Handles the logic
    pub async fn call_req(&mut self, request: Request) -> ResponseResult {
        let incoming_route = request.incoming_route();
        let outgoing = {
            let mut ret: Option<ResponseResult> = None;

            for stored in &mut self.routes {
                if stored.route.matches(&incoming_route) {
                    match (stored.func)(request.clone(), stored.route.clone()).await {
                        Outcome::Response(r) => {
                            ret = Some(r);
                            break;
                        },
                        Outcome::Forward => {
                            continue
                        }
                    }
                }
            }

            ret
        };

        // I wish async closures were stable
        match outgoing {
            Some(ret) => ret,
            None => default_no_route(&request).await
        }
    }
}

async fn default_no_route(_: &Request) -> ResponseResult {
    Ok(method_not_found())
}


#[cfg(test)]
mod tests {
    use futures::Future;
    use lambda_http::{
        Body,
        Context,
        http::Request as HttpRequest,
        IntoResponse,
        Response,
    };

    use crate::{
        application::{
            Application,
            RouteBuilder,
            SharedRoute,
        },
        response::RouteOutcome,
        request::Request,
        route::Route,
    };

    // Provides a shim which will always return 204
    async fn succ_shim(_: Request, _: SharedRoute) -> RouteOutcome {
        let ret = Response::builder()
            .status(204)
            .body(Body::Empty)
            .unwrap();

        RouteOutcome::Response(Ok(ret))
    }

    // Provides a shim which will always forward
    async fn forward_shim(_: Request, _: SharedRoute) -> RouteOutcome {
        RouteOutcome::Forward
    }

    // This is handled by code generation
    fn route_builder<'a, Func, Fut>(method: &'a str, uri: &'a str, func: Func) -> impl FnOnce() -> RouteBuilder<'a>
    where
        Func: (FnMut(Request, SharedRoute) -> Fut) + Send + 'static,
        Fut: Future<Output = RouteOutcome> + Send + 'static,
    {
        move || {
            RouteBuilder::new(
                Route::new(method, uri)
                    .unwrap(),
                func,
            )
        }
    }

    #[test]
    fn test_builder() {
        let app = Application::builder()
            .add_route(route_builder("GET", "/", succ_shim))
            .add_route(route_builder("GET", "/foo", succ_shim))
            .add_route(route_builder("GET", "/foo/bar/baz", forward_shim))
            .build()
            .unwrap();

        assert_eq!(app.routes.len(), 3);
    }

    #[tokio::test]
    async fn test_handler() {
        let mut app = Application::builder()
            .add_route(route_builder("GET", "/foo/bar", succ_shim))
            .add_route(route_builder("GET", "/bar/baz", forward_shim))
            .build()
            .unwrap();

        let lambda_req = HttpRequest::builder()
            .method("GET")
            .uri("http://api.test/foo/bar")
            .body(Body::Empty)
            .unwrap();

        let result = app.handle(
            lambda_req,
            Context::default()
        ).await
        .unwrap()
        .into_response();

        assert_eq!(result.status().as_u16(), 204);
        assert_eq!(app.routes.len(), 2);
    }

    #[tokio::test]
    async fn test_handler_default() {
        let mut app = Application::builder()
            .add_route(route_builder("GET", "/foo/bar", succ_shim))
            .add_route(route_builder("GET", "/bar/baz", forward_shim))
            .build()
            .unwrap();

        let lambda_req = HttpRequest::builder()
            .method("GET")
            .uri("http://api.test/bar/baz")
            .body(Body::Empty)
            .unwrap();

        let result = app.handle(
            lambda_req,
            Context::default()
        ).await
        .unwrap()
        .into_response();

        assert_eq!(result.status().as_u16(), 405);
        assert_eq!(app.routes.len(), 2);
    }
}


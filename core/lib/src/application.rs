use std::{
    sync::Arc,
};
use futures::{
    Future,
};
use state::Container;
use crate::{
    Context,
    error::OxideError,
    IntoResponse,
    LambdaRequest,
    outcome::Outcome,
    response::{
        method_not_found,
        ResponseError,
        ResponseResult,
        RouteOutcome,
    },
    request::OxideRequest,
};

use aws_oxide_api_route::Route;

pub type SharedRoute = Arc<Route>;
type RouteFunction = Box<dyn FnMut(OxideRequest, Arc<Route>) -> futures::future::BoxFuture<'static, RouteOutcome> + Send>;



// pub trait RouteFunction {

// }

pub static CONTAINER: Container = Container::new();

/// Route object which is stored and
/// associated with a handler function
struct StoredRoute {
    route: SharedRoute,
    func: RouteFunction,
}

pub struct ApplicationBuilder {
    routes: Vec<StoredRoute>,
    container: Container,
}

pub struct Application {
    routes: Vec<StoredRoute>,
    container: Arc<Container>,
}

pub struct RouteBuilder {
    route: Route,
    func: RouteFunction,
}

impl ApplicationBuilder {
    /// Returns an ApplicationBuilder.  This takes a function `func` which is expected
    /// to have been generated by the `route` macro.  See [`route`](aws_oxide_api_codegen::route) for more information.
    ///
    /// * `func` - Function which returns a RouteBuilder.  This should be a user created function decorated with the `route` macro.
    ///
    /// ```ignore
    /// fn main() {
    ///     Application::builder()
    ///         .add_route(hello)
    ///         .unwrap();
    /// }
    ///
    ///#[route("GET", "/some/:id")]
    ///async fn hello(id: i32) -> Result<impl IntoResponse, ResponseError> {
    ///    Ok(json!({"id": id}))
    ///}
    /// ```
    pub fn add_route<F>(mut self, func: F) -> Self
    where
        F: (FnOnce() -> RouteBuilder) + Send + 'static,
    {
        self.routes
            .push(func().into());

        self
    }

    pub fn build(self) -> Result<Application, OxideError> {
        let ret = Application::new(
            self.routes,
            self.container,
        );

        Ok(ret)
    }

    pub fn manage<T: Sync + Send + 'static>(self, data: T) -> Self {
        self.container.set(data);
        self
    }
}

impl RouteBuilder {
    pub fn new<Func, Fut>(route: Route, mut func: Func) -> Self
    where
        Func: (FnMut(OxideRequest, SharedRoute) -> Fut) + Send + 'static,
        Fut: Future<Output = RouteOutcome> + Send + 'static,
    {
        Self {
            route,
            func: Box::new(move |req: OxideRequest, route: SharedRoute| Box::pin(func(req, route))),
        }
    }
}

impl Into<StoredRoute> for RouteBuilder {
    fn into(self) -> StoredRoute {
        StoredRoute {
            route: Arc::new(self.route),
            func: self.func,
        }
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self {
            routes: Vec::new(),
            container: Container::new(),
        }
    }
}


impl Application {
    fn new(routes: Vec<StoredRoute>, container: Container) -> Self {
        Self {
            routes,
            container: Arc::new(container),
        }
    }

    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::default()
    }

    /// Entrypoint from Lambda main
    pub async fn handle(&mut self, event: LambdaRequest, _context: Context) -> Result<impl IntoResponse, ResponseError> {
        let request = OxideRequest::new(
            event,
            self
                .container
                .clone()
        );

        self.call_req(request).await
    }

    /// Handles the logic
    pub async fn call_req(&mut self, request: OxideRequest) -> ResponseResult {
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

async fn default_no_route(_: &OxideRequest) -> ResponseResult {
    Ok(method_not_found())
}


#[cfg(test)]
mod tests {
    use futures::Future;
    use crate::{
        application::{
            Application,
            RouteBuilder,
            SharedRoute,
        },
        Context,
        netlify_lambda_http::{
            Body,
            Response,
        },
        http::Request as HttpRequest,
        IntoResponse,
        response::RouteOutcome,
        request::OxideRequest,
        route::Route,
    };

    // Provides a shim which will always return 204
    async fn succ_shim(_: OxideRequest, _: SharedRoute) -> RouteOutcome {
        let ret = Response::builder()
            .status(204)
            .body(Body::Empty)
            .unwrap();

        RouteOutcome::Response(Ok(ret))
    }

    // Provides a shim which will always forward
    async fn forward_shim(_: OxideRequest, _: SharedRoute) -> RouteOutcome {
        RouteOutcome::Forward
    }

    // This is handled by code generation
    fn route_builder<'a, Func, Fut>(method: &'a str, uri: &'a str, func: Func) -> impl FnOnce() -> RouteBuilder
    where
        Func: (FnMut(OxideRequest, SharedRoute) -> Fut) + Send + 'static,
        Fut: Future<Output = RouteOutcome> + Send + 'static,
    {
        let builder = RouteBuilder::new(
            Route::new(
                method,
                uri,
            ).unwrap(),
            func,
        );

        move || { builder }
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


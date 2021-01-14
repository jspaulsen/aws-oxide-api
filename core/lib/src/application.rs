use std::sync::Arc;
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
    request::RouteRequest,
};

use aws_oxide_api_route::Route;

pub type SharedRoute = Arc<Route>;
pub type StoredRouteFunction = for<'a> fn(&'a RouteRequest, Arc<Route>) -> futures::future::BoxFuture<'a, RouteOutcome>;

pub static CONTAINER: Container = Container::new();

/// Route object which is stored and
/// associated with a handler function

pub struct StoredRoute {
    pub route: SharedRoute,
    pub func: StoredRouteFunction,
}

pub struct ApplicationBuilder {
    routes: Vec<StoredRoute>,
    container: Container,
}

pub struct Application {
    container: Container,
    routes: Vec<StoredRoute>,
}


impl ApplicationBuilder {
    /// Returns an ApplicationBuilder.  This takes a function `func` which is expected
    /// to have been generated by the `route` macro.  See [`route`](aws_oxide_api_codegen::route) for more information.
    ///
    /// * `func` - Function which returns a StoredRoute.  This should be a user created function decorated with the `route` macro.
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
        F: (FnOnce() -> StoredRoute)
    {
        self.routes
            .push(func());

        self
    }

    pub fn build(self) -> Result<Application, OxideError> {
        let ret = Application::new(
            self.routes,
            self.container,
        );

        Ok(ret)
    }

    /// Allows the API to "manage" data
    /// TODO: This is not useful + is an ugly hack requiring cloning data
    pub fn manage<T: Sync + Send + Clone + 'static>(self, data: T) -> Self {
        self.container.set(data);
        self
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
            container,
        }
    }

    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::default()
    }

    /// Entrypoint from Lambda main
    pub async fn handle(&'_ mut self, event: LambdaRequest, _context: Context) -> Result<impl IntoResponse, ResponseError> {
        let request = RouteRequest::new(
            event,
            &self.container,
        );

        let outgoing = {
            let mut ret: Option<ResponseResult> = None;
            let incoming_route = request
                .incoming_route();

            for stored in &mut self.routes {
                if stored.route.matches(&incoming_route) {
                    match (stored.func)(&request, stored.route.clone()).await {
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

        match outgoing {
            Some(ret) => ret,
            None => default_no_route(&request).await
        }
    }
}

async fn default_no_route(_: &RouteRequest<'_>) -> ResponseResult {
    Ok(method_not_found())
}


#[cfg(test)]
mod tests {
    use crate::{
        application::{
            Application,
            SharedRoute,
            StoredRoute,
            StoredRouteFunction,
        },
        Body,
        Context,
        netlify_lambda_http::{
            Response,
        },
        http::Request as HttpRequest,
        IntoResponse,
        response::RouteOutcome,
        request::RouteRequest,
        route::Route,
    };

    // Provides a shim which will always return 204
    fn succ_shim<'a>(request: &'a RouteRequest<'_>, route: SharedRoute) -> futures::future::BoxFuture<'a, RouteOutcome> {
        async fn inner_shim(_: &'_ RouteRequest<'_>, _: SharedRoute) -> RouteOutcome {
            let ret = Response::builder()
                .status(204)
                .body(Body::Empty)
                .unwrap();

            RouteOutcome::Response(Ok(ret))
        }

        Box::pin(inner_shim(request, route))
    }

    // Provides a shim which will always forward
    fn forward_shim<'a>(request: &'a RouteRequest<'_>, route: SharedRoute) -> futures::future::BoxFuture<'a, RouteOutcome> {
        async fn inner_shim(_: &'_ RouteRequest<'_>, _: SharedRoute) -> RouteOutcome {
            RouteOutcome::Forward
        }

        Box::pin(inner_shim(request, route))
    }

    // This is handled by code generation
    fn route_builder<'a>(method: &'a str, uri: &'a str, func: StoredRouteFunction) -> impl FnOnce() -> StoredRoute {
        let stored = StoredRoute {
            route: std::sync::Arc::new(
                Route::new(
                        method,
                        uri,
                ).unwrap()
            ),
            func,
        };

        move || { stored }
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


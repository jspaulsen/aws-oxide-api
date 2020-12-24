#[cfg(test)]
mod tests {
    use aws_oxide_api::{
        Application,
        Body,
        Context,
        http::Request as HttpRequest,
        IntoResponse,
        guards::Json,
        ResponseError,
        route,
    };

    use serde::Deserialize;
    use serde_json::json;
    use tokio;

    #[derive(Deserialize)]

    pub struct ExampleJson {
        field: String
    }

    #[tokio::test]
    async fn test_codegen_json() {
        #[route("POST", "/some/:id_a/another/:id_b")]
        fn route_test_fn(id_a: i32, id_b: String, body: Json<ExampleJson>) -> Result<impl IntoResponse, ResponseError> {
            let ret = json!({
                "id_a": id_a,
                "id_b": id_b,
                "field": body.field,
            });

            Ok(ret)
        }

        let mut app = Application::builder()
            .add_route(route_test_fn)
            .build()
            .expect("Application should build successfully");

        let expected_id_a: i32 = 52;
        let expected_id_b = "Hello";
        let expected_field = "this-is-a-field";
        let succ_uri = format!("http://api.test/some/{}/another/{}", expected_id_a, expected_id_b);
        let fail_uri = "http://api.test/some/123/another/abc";

        let succ_body = json!({"field": expected_field})
            .to_string();

        let fail_body = json!({"not-correct-field": expected_field})
            .to_string();

        let req_succ = HttpRequest::builder()
            .method("POST")
            .uri(succ_uri.clone())
            .header("Content-Type", "application/json")
            .body(Body::Text(succ_body.clone()))
            .expect("Request should build successfully");

        let req_fail_body = HttpRequest::builder()
            .method("POST")
            .uri(fail_uri)
            .header("Content-Type", "application/json")
            .body(Body::Text(fail_body))
            .expect("Request should build successfully");

        let req_fail_header = HttpRequest::builder()
            .method("POST")
            .uri(succ_uri)
            .body(Body::Text(succ_body))
            .expect("Request should build successfully");

        let result_succ = &app.handle(
            req_succ,
            Context::default(),
        )
        .await
        .unwrap()
        .into_response();

        let result_fail_body = &app.handle(
            req_fail_body,
            Context::default(),
        )
        .await
        .unwrap()
        .into_response();

        let result_fail_header = &app.handle(
            req_fail_header,
            Context::default(),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(result_succ.status().as_u16(), 200);
        assert_eq!(result_fail_body.status().as_u16(), 400);
        assert_eq!(result_fail_header.status().as_u16(), 400);

        let returned_body = match result_succ.body() {
            Body::Text(body) => {
                body
            },
            _ => {
                assert!(false, "Only Text should be returned");
                unreachable!()
            }
        };

        let body_json: serde_json::Value = serde_json::from_str(returned_body)
            .expect("Body should be deserializable JSON");

        assert_eq!(body_json["id_a"], expected_id_a);
        assert_eq!(body_json["id_b"], expected_id_b);
        assert_eq!(body_json["field"], expected_field);
    }

//     let lambda_req = HttpRequest::builder()
//     .method("GET")
//     .uri("http://api.test/foo/bar")
//     .body(Body::Empty)
//     .unwrap();

// let result = app.handle(
//     lambda_req,
//     Context::default()
// ).await
// .unwrap()
// .into_response();

// assert_eq!(result.status().as_u16(), 204);
// assert_eq!(app.routes.len(), 2);
// }
}



// pub struct RouteBuilder<'a> {
//     route: Route,
//     func: RouteFunction<'a>,
// }
// AnotherType

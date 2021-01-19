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
        State,
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

    #[derive(Clone)]
    pub struct Something {
        a_field: String
    }

    #[tokio::test]
    async fn test_codegen_state() {
        let expected_field = "abcd";
        #[route("POST", "/some/field")]
        fn route_test_fn(state: State<Something>) -> Result<impl IntoResponse, ResponseError> {
            let ret = json!({
                "a_field": &state.a_field,
            });

            Ok(ret)
        }

        let mut app = Application::builder()
            .add_route(route_test_fn)
            .manage(Something {a_field: expected_field.into()})
            .build()
            .expect("Application should build successfully");

        let req = HttpRequest::builder()
            .method("POST")
            .uri("/some/field")
            .header("Content-Type", "application/json")
            .body(Body::Empty)
            .expect("Request should build successfully");

        let result = &app.handle(req, Context::default())
            .await
            .unwrap()
            .into_response();

        let body_json = match result.body() {
            Body::Text(body) => {
                body
            },
            _ => {
                assert!(false, "Only Text should be returned");
                unreachable!()
            }
        };

        let result_payload: serde_json::Value = serde_json::from_str(body_json)
            .expect("Body should be deserializable JSON");

        assert_eq!(result_payload["a_field"], expected_field);
    }

    #[tokio::test]
    async fn test_return_type() {
        #[route("GET", "/some/:id_a")]
        async fn into_return_type(id_a: i32) -> serde_json::Value {
            json!({"id_a": id_a})
        }

        let mut app = Application::builder()
            .add_route(into_return_type)
            .build()
            .expect("Application should build successfully");

        let expected_id: i32 = 12345;
        let uri = format!("/some/{}", expected_id);

        let req = HttpRequest::builder()
            .method("GET")
            .uri(uri)
            .body(Body::Empty)
            .expect("Request should build successfully");

        let result = &app.handle(req, Context::default())
            .await
            .unwrap()
            .into_response();

        let body_json = match result.body() {
            Body::Text(body) => {
                body
            },
            _ => {
                assert!(false, "Only Text should be returned");
                unreachable!()
            }
        };

        let result_payload: serde_json::Value = serde_json::from_str(body_json)
            .expect("Body should be deserializable JSON");

        assert_eq!(result_payload["id_a"], expected_id)
    }
}

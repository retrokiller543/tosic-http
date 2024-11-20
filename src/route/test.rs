//! Tests for route

#![cfg(test)]

use super::*;
use crate::request::HttpRequest;
use crate::response::HttpResponse;

#[test]
fn test_static_match() {
    let route = Route::new("/echo/test");

    assert!(route.is_match("/echo/test").is_some());
    assert!(route.is_match("/echo/fail").is_none());
}

#[test]
fn test_parameter_match() {
    let route = Route::new("/echo/{message}");
    let params = route.is_match("/echo/hello").unwrap();
    assert_eq!(params.get("message"), Some(&"hello".to_string()));
}

#[test]
fn test_wildcard_match() {
    let route = Route::new("/echo/*");
    assert!(route.is_match("/echo/anything").is_some());
    assert!(route.is_match("/echo/multiple/segments").is_none());
}

#[test]
fn test_wildcard_deep_match() {
    let route = Route::new("/echo/**");
    let params = route.is_match("/echo/multiple/segments").unwrap();
    assert_eq!(
        params.get("wildcard_deep"),
        Some(&"multiple/segments".to_string())
    );
}

#[test]
fn test_route_addition() {
    let route1 = Route::new("/echo");
    let route2 = Route::new("/test");
    let combined = route1 + route2;
    assert!(combined.is_match("/echo/test").is_some());
}

#[test]
fn test_route_ordering() {
    let route1 = Route::new("/echo");
    let route2 = Route::new("/echo/{message}");
    assert!(route1 < route2);
}

#[test]
fn test_route_iteration() {
    let route = Route::new("/echo/{message}");
    let segments: Vec<_> = route.into_iter().collect();
    assert_eq!(segments.len(), 2);
}

#[test]
fn test_match_wildcard_path() {
    let mut root = RouteNode::new();
    root.insert(&Route::new("/wildcard/*"), |_req: HttpRequest| async move {
        HttpResponse::new(200)
    });
    let route = Route::new("/wildcard/some_value");
    assert!(root.match_path(&route).is_some());
}

#[test]
fn test_match_deep_wildcard_path() {
    let mut root = RouteNode::new();
    root.insert(
        &Route::new("/wildcard_deep/**"),
        |_req: HttpRequest| async move { HttpResponse::new(200) },
    );
    let route = Route::new("/wildcard_deep/any/level/of/segments");
    assert!(root.match_path(&route).is_some());
}

/*#[tokio::test]
async fn test_route_json_handler() {
    let route = Route::new("/echo/{message}");
    assert!(route.is_match("/echo/test").is_some());

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStruct {
        test: String,
    }

    async fn json_handler(json: Json<TestStruct>) -> impl Responder<Body = BoxBody> {
        let data = json.into_inner();
        assert_eq!(data.test, "test");

        data.test
    }

    let mut node = RouteNode::new();
    node.insert(&route, json_handler);

    let test_route = Route::new("/echo/test");
    let handler = node.match_path(&test_route);
    assert!(handler.is_some());

    let (handler, params) = handler.unwrap();

    assert_eq!(params.get("message"), Some(&"test".to_string()));
    let req = HttpRequest::new(
        Method::GET,
        Uri::from_static("/echo/test"),
        HeaderMap::new(),
        Version::HTTP_11,
    );

    let payload = br#"{ "test": "test" }"#;
    let req = HttpRequest::new(
        Method::POST,
        Uri::from_static("/echo/test"),
        HeaderMap::new(),
        Version::HTTP_11,
    );

    let mut payload_struct = HttpPayload::from_bytes(payload.try_into_bytes().unwrap());

    let res: Result<HttpResponse, Error> = handler.call((req, &mut payload_struct)).await;

    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res.status_code, 200);
}

#[tokio::test]
async fn test_route_node_insertion() {
    let mut node = RouteNode::new();

    async fn test_handler(req: HttpRequest) -> impl Responder<Body = BoxBody> {
        "test"
    }

    let route = Route::new("/echo/{message}");
    node.insert(&route, test_handler);

    let test_route = Route::new("/echo/test");

    let handler = node.match_path(&test_route);
    assert!(handler.is_some());

    let (handler, params) = handler.unwrap();

    assert_eq!(params.get("message"), Some(&"test".to_string()));
    let req = HttpRequest::new(
        Method::GET,
        Uri::from_static("/echo/test"),
        HeaderMap::new(),
        Version::HTTP_11,
    );

    let res: Result<HttpResponse, Error> = handler.call((req, &mut HttpPayload::default())).await;

    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res.status_code, 200);
}*/

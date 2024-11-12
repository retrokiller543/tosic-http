#[cfg(test)]
use super::*;

#[test]
fn test_http_request_new() {
    let method = Method::GET;
    let uri: Uri = "/test".parse().unwrap();
    let headers = HeaderMap::new();
    let version = Version::HTTP_11;

    let request = HttpRequest::new(method.clone(), uri.clone(), headers.clone(), version);

    assert_eq!(request.method, method);
    assert_eq!(request.uri, uri);
    assert_eq!(request.headers, headers);
    assert_eq!(request.version, version);
    assert!(request.params.is_none());
}

#[test]
fn test_http_request_from_bytes_complete() {
    let bytes = b"GET /test HTTP/1.1\r\nHost: example.com\r\nUser-Agent: test-agent\r\n\r\n";
    //let bytes = Bytes::copy_from_slice(bytes);

    let (request, _) = HttpRequest::from_bytes(bytes).unwrap();

    assert_eq!(request.method, Method::GET);
    assert_eq!(request.uri, "/test".parse::<Uri>().unwrap());
    assert_eq!(request.version, Version::HTTP_11);

    let host_header = request.headers.get("Host").unwrap();
    assert_eq!(host_header, "example.com");

    let user_agent_header = request.headers.get("User-Agent").unwrap();
    assert_eq!(user_agent_header, "test-agent");
}

#[test]
fn test_http_request_from_bytes_incomplete() {
    let bytes = b"GET /test HTTP/1.1\r\nHost: example.com";
    //let bytes = Bytes::copy_from_slice(bytes);

    let result = HttpRequest::from_bytes(bytes);
    assert!(result.is_err());
}

#[test]
fn test_from_request() {
    let method = Method::POST;
    let uri: Uri = "/test".parse().unwrap();
    let headers = HeaderMap::new();
    let version = Version::HTTP_11;

    let req = HttpRequest::new(method, uri, headers, version);
    let future = HttpRequest::from_request(&req, &mut HttpPayload::default());

    let result = futures::executor::block_on(future).unwrap();
    assert_eq!(result.method, Method::POST);
    assert_eq!(result.uri, "/test".parse::<Uri>().unwrap());
    assert_eq!(result.version, Version::HTTP_11);
}

use crate::body::BoxBody;

pub struct HttpRequest {
    pub method: http::Method,
    pub uri: http::Uri,
    pub headers: http::HeaderMap,
    pub version: http::Version,
}

pub struct HttpPayload(BoxBody);

use crate::body::BoxBody;
use crate::futures::{ok, Ready};
use crate::traits::from_request::FromRequest;
use std::collections::HashMap;
use std::convert::Infallible;

#[derive(Clone, Debug)]
pub struct HttpRequest {
    pub method: http::Method,
    pub uri: http::Uri,
    pub headers: http::HeaderMap,
    pub version: http::Version,
    pub params: Option<HashMap<String, String>>,
}

pub struct HttpPayload(BoxBody);

impl HttpRequest {
    pub fn new(
        method: http::Method,
        uri: http::Uri,
        headers: http::HeaderMap,
        version: http::Version,
    ) -> Self {
        HttpRequest {
            method,
            uri,
            headers,
            version,
            params: None,
        }
    }
}

impl FromRequest for HttpRequest {
    type Error = Infallible;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest) -> Self::Future {
        ok(req.clone())
    }
}

/*impl<'a> FromRequest for &'a HttpRequest {
    type Error = Infallible;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &'a HttpRequest) -> Self::Future {
        ok(req)
    }
}*/

mod test;

use crate::body::message_body::MessageBody;
use crate::body::BoxBody;
use crate::error::ServerError;
use crate::futures::{ok, Ready};
use crate::state::State;
use crate::traits::from_request::FromRequest;
use bytes::Bytes;
use http::{HeaderMap, HeaderValue, Method, Uri, Version};
use httparse::{Request, Status};
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

#[derive(Clone, Debug, Default)]
pub struct HttpRequest {
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap,
    pub version: Version,
    pub params: BTreeMap<String, String>,
    pub data: State,
}

#[derive(Clone, Debug)]
pub struct HttpPayload(BoxBody);

impl Default for HttpPayload {
    fn default() -> Self {
        HttpPayload::new(BoxBody::new(()))
    }
}

impl HttpPayload {
    pub(crate) fn new(body: BoxBody) -> Self {
        HttpPayload(body)
    }

    pub(crate) fn from_bytes(bytes: Bytes) -> Self {
        HttpPayload(BoxBody::new(bytes))
    }
}

impl Deref for HttpPayload {
    type Target = BoxBody;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HttpPayload {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl HttpRequest {
    pub(crate) fn new(method: Method, uri: Uri, headers: HeaderMap, version: Version) -> Self {
        HttpRequest {
            method,
            uri,
            headers,
            version,
            ..Default::default()
        }
    }

    pub(crate) fn from_bytes(buffer: &[u8]) -> Result<(Self, HttpPayload), ServerError> {
        let mut headers = [httparse::EMPTY_HEADER; 32];
        let mut req = Request::new(&mut headers);

        match req.parse(buffer) {
            Ok(Status::Complete(_)) => {
                let parsed_req: Self = req.into();

                // Extract body
                let headers_end = buffer
                    .windows(4)
                    .position(|window| window == b"\r\n\r\n")
                    .map(|pos| pos + 4)
                    .unwrap_or(buffer.len());
                let body = buffer[headers_end..].to_vec();

                Ok((
                    parsed_req,
                    HttpPayload::from_bytes(body.try_into_bytes().unwrap()),
                ))
            }
            Ok(Status::Partial) => Err(ServerError::PartialParsed),
            Err(e) => Err(ServerError::ParseError(e)),
        }
    }

    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    pub fn path(&self) -> &str {
        self.uri().path()
    }

    pub fn query(&self) -> Option<&str> {
        self.uri().query()
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn params(&self) -> &BTreeMap<String, String> {
        &self.params
    }

    pub fn params_mut(&mut self) -> &mut BTreeMap<String, String> {
        &mut self.params
    }
}

impl FromRequest for HttpRequest {
    type Error = Infallible;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut HttpPayload) -> Self::Future {
        ok(req.clone())
    }
}

impl From<Request<'_, '_>> for HttpRequest {
    fn from(value: Request) -> Self {
        let version = match value.version.unwrap_or(0) {
            0 => Version::HTTP_10,
            1 => Version::HTTP_11,
            _ => Version::HTTP_11,
        };
        let method: Method = value.method.unwrap_or("GET").parse().unwrap_or_default();
        let uri = Uri::from_str(value.path.unwrap_or_default()).unwrap_or_default();
        let mut headers = HeaderMap::new();

        for header in value.headers {
            // Parse header name into an owned HeaderName
            let header_name = header.name.parse::<http::header::HeaderName>().unwrap();

            // Create HeaderValue from bytes (this clones the data)
            let header_value = HeaderValue::from_bytes(header.value).unwrap();

            headers.append(header_name, header_value);
        }

        Self {
            method,
            uri,
            headers,
            version,
            ..Default::default()
        }
    }
}

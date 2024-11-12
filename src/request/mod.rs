mod test;

use crate::body::BoxBody;
use crate::futures::{ok, Ready};
use crate::traits::from_request::FromRequest;
use bytes::Bytes;
use http::{HeaderMap, HeaderValue, Method, Uri, Version};
use httparse::{Request, Status};
use std::collections::HashMap;
use std::convert::Infallible;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub struct HttpRequest {
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap,
    pub version: Version,
    pub params: Option<HashMap<String, String>>,
    //pub payload: HttpPayload
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
            params: None,
            //payload: HttpPayload::new(BoxBody::new(())),
        }
    }

    pub(crate) fn from_bytes(
        buffer: &'static [u8],
    ) -> Result<(Self, HttpPayload), Box<dyn std::error::Error>> {
        let mut headers = [httparse::EMPTY_HEADER; 32];
        let mut req = Request::new(&mut headers);

        match req.parse(buffer)? {
            Status::Complete(_len) => {
                let method = req.method.ok_or("No method")?.parse::<Method>()?;
                let uri = req.path.ok_or("No URI")?.parse::<Uri>()?;
                let version = match req.version.ok_or("No version")? {
                    0 => Version::HTTP_10,
                    1 => Version::HTTP_11,
                    _ => Version::HTTP_11,
                };

                let mut header_map = HeaderMap::new();
                for header in req.headers.iter() {
                    let name = header.name;
                    let value = HeaderValue::from_bytes(header.value)?;
                    header_map.insert(name, value);
                }

                let parsed_req = HttpRequest::new(method, uri, header_map, version);

                // Find the end of the headers, and copy the payload part.
                let headers_end = buffer
                    .windows(4)
                    .position(|window| window == b"\r\n\r\n")
                    .map(|pos| pos + 4)
                    .unwrap_or(buffer.len());

                // Here, we clone the data to ensure the payload is self-contained and owns the data.
                let payload_data = buffer[headers_end..].to_vec();
                let payload = HttpPayload::from_bytes(payload_data.into());

                Ok((parsed_req, payload))
            }
            Status::Partial => Err("Incomplete HTTP request".into()),
        }
    }
}

impl FromRequest for HttpRequest {
    type Error = Infallible;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut HttpPayload) -> Self::Future {
        ok(req.clone())
    }
}

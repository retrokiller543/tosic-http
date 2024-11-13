use crate::body::message_body::MessageBody;
use crate::body::BoxBody;
use crate::request::HttpRequest;
use crate::traits::responder::Responder;
use http::StatusCode;
use std::fmt::Debug;
use std::io::Write;
use tokio::io;

#[derive(Clone, Debug)]
pub struct HttpResponse<Body = BoxBody> {
    pub(crate) body: Body,
    pub(crate) status_code: StatusCode,
    pub(crate) headers: http::HeaderMap,
    pub(crate) version: http::Version,
}

impl HttpResponse<BoxBody> {
    pub fn new<T: TryInto<StatusCode>>(status_code: T) -> Self
    where
        T::Error: Debug,
    {
        Self {
            body: BoxBody::new(()),
            status_code: status_code.try_into().unwrap(),
            headers: http::HeaderMap::new(),
            version: http::Version::HTTP_11,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        self.status_code
    }

    pub fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut http::HeaderMap {
        &mut self.headers
    }

    pub fn set_body(self, body: BoxBody) -> Self {
        Self { body, ..self }
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut response_bytes = Vec::new();

        let status_line = format!(
            "{:?} {} {}\r\n",
            self.version,
            self.status_code.as_str(),
            self.status_code.canonical_reason().unwrap_or("Unknown")
        );

        response_bytes.write_all(status_line.as_bytes())?;

        for (key, value) in &self.headers {
            let header_line = format!("{}: {}\r\n", key, value.to_str().unwrap());
            response_bytes.write_all(header_line.as_bytes())?;
        }

        response_bytes.write_all(b"\r\n")?;

        let body = self.clone().body.try_into_bytes().unwrap_or_default(); // TODO: There is probably a better way to handle this

        response_bytes.write_all(&body)?;

        Ok(response_bytes)
    }

    pub fn body<B>(mut self, body: B) -> Self
    where
        B: MessageBody + Clone + 'static
    {
        self.body = BoxBody::new(body);

        self
    }
}

impl Responder for HttpResponse<BoxBody> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        self
    }
}

impl Responder for String {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::new(200).set_body(BoxBody::new(self))
    }
}

impl Responder for &'static str {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::new(200).set_body(BoxBody::new(self))
    }
}

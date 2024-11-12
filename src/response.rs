use crate::body::BoxBody;
use crate::request::HttpRequest;
use crate::traits::responder::Responder;
use http::StatusCode;
use std::fmt::Debug;

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

use crate::body::BoxBody;
use crate::error::response_error::ResponseError;
use crate::response::HttpResponse;
use http::{HeaderMap, Response};
use std::convert::Infallible;
use std::fmt;
use std::fmt::Debug;
use thiserror::Error;

mod foreign_impls;
pub mod macros;
pub mod response_error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Error(#[from] Error),
    #[error(transparent)]
    Http(#[from] http::Error),
    #[error(transparent)]
    ExtractionError(#[from] crate::extractors::ExtractionError),
}

pub struct Error {
    cause: Box<dyn ResponseError>,
}

/*impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.cause, f)
    }
}*/

impl Error {
    /// Returns the reference to the underlying `ResponseError`.
    pub fn as_response_error(&self) -> &dyn ResponseError {
        self.cause.as_ref()
    }

    /// Similar to `as_response_error` but downcasts.
    pub fn as_error<T: ResponseError + 'static>(&self) -> Option<&T> {
        <dyn ResponseError>::downcast_ref(self.cause.as_ref())
    }

    /// Shortcut for creating an `HttpResponse`.
    pub fn error_response(&self) -> HttpResponse {
        self.cause.error_response()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.cause, f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self.cause)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// `Error` for any error that implements `ResponseError`
impl<T: ResponseError + 'static> From<T> for Error {
    fn from(err: T) -> Error {
        Error {
            cause: Box::new(err),
        }
    }
}

impl From<Box<dyn ResponseError>> for Error {
    fn from(value: Box<dyn ResponseError>) -> Self {
        Error { cause: value }
    }
}

impl From<Error> for Response<BoxBody> {
    fn from(err: Error) -> Response<BoxBody> {
        err.error_response().into()
    }
}

impl From<HttpResponse> for Response<BoxBody> {
    fn from(value: HttpResponse<BoxBody>) -> Self {
        let mut response = Response::builder().status(value.status_code());

        let headers = if let Some(headers) = response.headers_mut() {
            headers
        } else {
            &mut HeaderMap::new()
        };

        for (header, value) in value.headers() {
            headers.insert(header, value.clone());
        }

        response.body(value.body).unwrap()
    }
}

impl From<Infallible> for Error {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

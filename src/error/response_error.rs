//! The `ResponseError` trait

use crate::body::BoxBody;
use crate::error::macros::{downcast_dyn, downcast_get_type_id};
use crate::response::HttpResponse;
use bytes::BytesMut;
use http::StatusCode;

use crate::error::ServerError;
use crate::extractors::ExtractionError;
use std::io::Write;

#[diagnostic::on_unimplemented(
    message = "`{Self}` cant be sent back as an error when used as a Handler",
    label = "Implement `ResponseError` for `{Self}` to send it back as an error",
    note = "any type that implements the trait `ResponseError` can be used as a result like this `Result<impl Responder<Body = BoxBody>, {Self}>`"
)]
/// # ResponseError
///
/// The `ResponseError` trait is used to define how an error looks like when sent back to the client
///
pub trait ResponseError: std::fmt::Debug + std::fmt::Display + Send {
    /// The status code to be sent back to the client
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    /// Shortcut for creating an `HttpResponse`
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let mut response = HttpResponse::new(self.status_code());

        let mut buff = BytesMut::new();
        let _ = write!(crate::utils::MutWriter(&mut buff), "{}", self);

        let mime = mime::TEXT_PLAIN_UTF_8.to_string();

        response
            .headers_mut()
            .insert(http::header::CONTENT_TYPE, mime.parse().unwrap());

        response.set_body(BoxBody::new(buff.freeze()))
    }

    downcast_get_type_id!();
}

downcast_dyn!(ResponseError);

impl ResponseError for ServerError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServerError::ExtractionError(err) => err.status_code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ResponseError for ExtractionError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

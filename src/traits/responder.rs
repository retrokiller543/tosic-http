//! This module contains the [`Responder`] trait which allows for returning a response from a handler.

use crate::body::message_body::MessageBody;
use crate::body::BoxBody;
use crate::error::response_error::ResponseError;
use crate::request::HttpRequest;
use crate::response::HttpResponse;

#[diagnostic::on_unimplemented(
    message = "Either implement responder for `{Self}` to return it directly or use `HttpResponse`",
    note = "you can use the more flexible approach for return types by using `impl Responder<Body = tosic_http::body::BoxBody>` this will allow you to return any type that implements `Responder`",
    note = "due to limits with the Rust type system you can only return one type in a function"
)]
/// # Responder
///
/// The `Responder` trait allows for returning a response from a handler.
pub trait Responder<Body = BoxBody> {
    /// The body of the response
    type Body: MessageBody + 'static;

    /// Returns the response for the request
    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body>;
}

impl<T, E> Responder for Result<T, E>
where
    T: Responder<Body = BoxBody>,
    E: ResponseError,
{
    type Body = BoxBody;

    #[inline]
    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        match self {
            Ok(okay) => okay.respond_to(req),
            Err(error) => error.error_response(),
        }
    }
}

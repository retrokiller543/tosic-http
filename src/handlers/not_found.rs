//! The default not found handler

use crate::body::BoxBody;
use crate::request::HttpRequest;
use crate::response::HttpResponse;
use crate::traits::responder::Responder;

pub async fn not_found(_req: HttpRequest) -> impl Responder<Body = BoxBody> {
    const NOT_FOUND_PAGE: &str = "Not Found";

    HttpResponse::new(404).set_body(BoxBody::new(NOT_FOUND_PAGE))
}

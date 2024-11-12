use crate::body::message_body::MessageBody;
use crate::body::BoxBody;
use crate::request::HttpRequest;
use crate::response::HttpResponse;

pub trait Responder {
    type Body: MessageBody + 'static = BoxBody;
    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body>;
}

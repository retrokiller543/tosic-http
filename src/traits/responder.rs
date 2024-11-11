use crate::body::message_body::MessageBody;
use crate::request::HttpRequest;
use crate::response::HttpResponse;

pub trait Responder {
    type Body: MessageBody + 'static;
    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body>;
}

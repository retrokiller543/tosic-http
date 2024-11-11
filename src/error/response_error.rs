use crate::body::BoxBody;
use crate::error::macros::{downcast_dyn, downcast_get_type_id};
use crate::response::HttpResponse;
use bytes::BytesMut;
use http::StatusCode;

use std::io::Write;

pub trait ResponseError: std::fmt::Debug + std::fmt::Display {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

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

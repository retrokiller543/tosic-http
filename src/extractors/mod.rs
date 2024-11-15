use crate::error::response_error::ResponseError;
use crate::request::{HttpPayload, HttpRequest};
use crate::traits::from_request::FromRequest;
use std::convert::Infallible;
use std::future::Future;
use thiserror::Error;

pub mod json;
pub mod query;

#[derive(Debug, Error)]
pub enum ExtractionError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("Failed to parse query: {0}")]
    Query(#[from] serde::de::value::Error),
}

impl<E> FromRequest for Option<E>
where
    E: FromRequest + Send,
    E::Future: Send + 'static,
{
    type Error = Infallible;
    type Future = impl Future<Output = Result<Option<E>, Self::Error>> + Send;

    fn from_request(req: &HttpRequest, payload: &mut HttpPayload) -> Self::Future {
        let future = E::from_request(req, payload);

        async move {
            match future.await {
                Ok(value) => Ok(Some(value)),
                Err(_) => Ok(None),
            }
        }
    }
}

// FromRequest implementation for Result<E, Err>
impl<E, Err> FromRequest for Result<E, Err>
where
    E: FromRequest + Send,
    E::Future: Send + 'static,
    Err: ResponseError + From<E::Error> + Send,
{
    type Error = Infallible;
    type Future = impl Future<Output = Result<Result<E, Err>, Self::Error>> + Send;

    fn from_request(req: &HttpRequest, payload: &mut HttpPayload) -> Self::Future {
        let future = E::from_request(req, payload);

        async move {
            match future.await {
                Ok(value) => Ok(Ok(value)),
                Err(e) => Ok(Err(Err::from(e))),
            }
        }
    }
}

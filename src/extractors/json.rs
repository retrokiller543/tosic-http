use crate::body::message_body::MessageBody;
use crate::body::BoxBody;
use crate::error::ServerError;
use crate::extractors::ExtractionError;
use crate::futures::{err, ok, Ready};
use crate::traits::from_request::FromRequest;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Json<V: DeserializeOwned + Serialize + Debug>(pub V);

impl<V: Serialize + Send + for<'de> serde::Deserialize<'de> + Debug> FromRequest for Json<V> {
    type Error = ServerError;
    type Future = Ready<Result<Json<V>, Self::Error>>;
    fn from_request(
        _: &crate::request::HttpRequest,
        payload: &mut crate::request::HttpPayload,
    ) -> Self::Future {
        let body = <BoxBody as Clone>::clone(payload)
            .boxed()
            .try_into_bytes()
            .expect("Unable to read body");

        match serde_json::from_slice(&body) {
            Ok(value) => ok(Json(value)),
            Err(error) => err(ServerError::ExtractionError(ExtractionError::Json(error))),
        }
    }
}

impl<T: DeserializeOwned + Serialize + Debug> Json<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: DeserializeOwned + Serialize + Debug> std::ops::Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: DeserializeOwned + Serialize + Debug> std::ops::DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

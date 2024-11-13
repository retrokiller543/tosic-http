use crate::extractors::ExtractionError;
use crate::futures::{err, ok, Ready};
use crate::request::{HttpPayload, HttpRequest};
use crate::traits::from_request::FromRequest;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path<V>(pub V);

impl<T> Path<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Path<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Path<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<V: DeserializeOwned> Path<V> {
    pub fn from_params(params: &HashMap<String, String>) -> Result<Self, ExtractionError> {
        // Convert the HashMap to a query string
        let query_string =
            serde_urlencoded::to_string(params).map_err(ExtractionError::QuerySerialize)?;

        // Deserialize the query string into the desired type
        serde_urlencoded::from_str::<V>(&query_string)
            .map(Self)
            .map_err(ExtractionError::Query)
    }
}

impl<V: DeserializeOwned> FromRequest for Path<V> {
    type Error = ExtractionError;
    type Future = Ready<Result<Path<V>, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut HttpPayload) -> Self::Future {
        // Extract the path parameters from the request
        let params = req.params();

        match Self::from_params(params) {
            Ok(path) => ok(path),
            Err(error) => err(error),
        }
    }
}

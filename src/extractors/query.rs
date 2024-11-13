use crate::extractors::ExtractionError;
use crate::futures::{err, ok, Ready};
use crate::request::{HttpPayload, HttpRequest};
use crate::traits::from_request::FromRequest;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Query<V>(pub V);

impl<T> Query<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: DeserializeOwned> Query<T> {
    pub fn from_query(query: &str) -> Result<Self, ExtractionError> {
        serde_urlencoded::from_str::<T>(query)
            .map(Self)
            .map_err(ExtractionError::Query)
    }
}

impl<T> std::ops::Deref for Query<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Query<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<V: DeserializeOwned> FromRequest for Query<V> {
    type Error = ExtractionError;
    type Future = Ready<Result<Query<V>, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut HttpPayload) -> Self::Future {
        let query = req.uri().query().unwrap_or("");

        match Self::from_query(query) {
            Ok(query) => ok(query),
            Err(error) => err(error),
        }
    }
}

use crate::extractors::ExtractionError;
use crate::futures::{err, ok, Ready};
use crate::request::{HttpPayload, HttpRequest};
use crate::traits::from_request::FromRequest;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone)]
pub struct Data<T: Send + Sync + 'static>(pub Arc<T>);

impl<T: Send + Sync + 'static> Data<T> {
    pub(crate) fn new(data: Arc<T>) -> Self {
        Data(data)
    }
    pub fn into_inner(self) -> Arc<T> {
        Arc::clone(&self.0)
    }
}

impl<T: Send + Sync + 'static> FromRequest for Data<T> {
    type Error = ExtractionError;
    type Future = Ready<Result<Data<T>, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut HttpPayload) -> Self::Future {
        let data = &req.data;

        match data.get::<T>() {
            Some(state) => ok(Data::new(state)),
            None => err(ExtractionError::DataNotFound),
        }
    }
}

impl<T: Send + Sync + 'static> Deref for Data<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Send + Sync + 'static> Debug for Data<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data").finish()
    }
}

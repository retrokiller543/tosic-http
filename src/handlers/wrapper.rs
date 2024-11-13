use crate::route::HandlerFn;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct HandlerWrapper(
    pub(crate) Arc<HandlerFn>,
    pub(crate) BTreeMap<String, String>,
);

impl HandlerWrapper {
    pub fn handler(&self) -> Arc<HandlerFn> {
        self.0.clone()
    }

    pub fn params(&self) -> &BTreeMap<String, String> {
        &self.1
    }
}

impl From<(Arc<HandlerFn>, BTreeMap<String, String>)> for HandlerWrapper {
    fn from(value: (Arc<HandlerFn>, BTreeMap<String, String>)) -> Self {
        Self(value.0, value.1)
    }
}

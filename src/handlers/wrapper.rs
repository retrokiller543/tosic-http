use crate::route::HandlerFn;
use std::collections::BTreeMap;

pub struct HandlerWrapper(pub(crate) HandlerFn, pub(crate) BTreeMap<String, String>);

impl HandlerWrapper {
    pub fn handler(&self) -> HandlerFn {
        self.0.clone()
    }

    pub fn params(&self) -> &BTreeMap<String, String> {
        &self.1
    }
}

impl From<(HandlerFn, BTreeMap<String, String>)> for HandlerWrapper {
    fn from(value: (HandlerFn, BTreeMap<String, String>)) -> Self {
        Self(value.0, value.1)
    }
}

use crate::route::HandlerFn;
use std::collections::HashMap;
use std::sync::Arc;

pub struct HandlerWrapper(pub(crate) Arc<HandlerFn>, pub(crate) HashMap<String, String>);

impl HandlerWrapper {
    pub fn handler(&self) -> Arc<HandlerFn> {
        self.0.clone()
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.1
    }
}

impl From<(Arc<HandlerFn>, HashMap<String, String>)> for HandlerWrapper {
    fn from(value: (Arc<HandlerFn>, HashMap<String, String>)) -> Self {
        Self(value.0, value.1)
    }
}

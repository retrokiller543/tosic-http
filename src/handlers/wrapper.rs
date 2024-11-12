use crate::route::HandlerFn;
use std::collections::HashMap;
use std::sync::Arc;

pub struct HandlerWrapper(Arc<HandlerFn>, HashMap<String, String>);

impl From<(Arc<HandlerFn>, HashMap<String, String>)> for HandlerWrapper {
    fn from(value: (Arc<HandlerFn>, HashMap<String, String>)) -> Self {
        Self(value.0, value.1)
    }
}

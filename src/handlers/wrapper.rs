//! Wraps a handler and a map of parameters to make it simpler to work with in other parts of the project

use crate::route::HandlerFn;
use std::collections::BTreeMap;

/// Wraps a handler and a map of parameters
pub struct HandlerWrapper(pub(crate) HandlerFn, pub(crate) BTreeMap<String, String>);

impl HandlerWrapper {
    /// Returns the handler
    pub fn handler(&self) -> HandlerFn {
        self.0.clone()
    }
}

impl From<(HandlerFn, BTreeMap<String, String>)> for HandlerWrapper {
    fn from(value: (HandlerFn, BTreeMap<String, String>)) -> Self {
        Self(value.0, value.1)
    }
}

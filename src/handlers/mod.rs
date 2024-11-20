//! Stores the handlers for each route. and keyed by method and then stored in a tree.

mod not_found;
pub(crate) mod wrapper;

use crate::body::BoxBody;
use crate::error::Error;
use crate::handlers::not_found::not_found;
use crate::handlers::wrapper::HandlerWrapper;
use crate::route::{HandlerFn, Route, RouteNode};
use crate::traits::from_request::FromRequest;
use crate::traits::handler::Handler;
use crate::traits::responder::Responder;
use http::Method;
use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use tracing::debug;

#[derive(Default, Debug, Clone)]
/// A collection of handlers for each route.
pub struct Handlers(pub HashMap<Method, RouteNode>);

impl Handlers {
    /// Create a new empty collection
    pub fn new() -> Self {
        Handlers(HashMap::new())
    }

    /// Insert a handler for a route and method
    pub fn insert<H, Args>(&mut self, method: Method, path: &str, handler: H)
    where
        H: Handler<Args> + Send + Sync + 'static,
        Args: FromRequest + Send + 'static,
        Args::Future: Future + Send + 'static,
        H::Future: Future + Send + 'static,
        H::Output: Responder<Body = BoxBody> + 'static,
        Error: From<Args::Error>,
    {
        let entry = self.entry(method).or_default();
        let route = Route::new(path);
        entry.insert(&route, handler);
    }

    #[cfg_attr(feature = "trace", tracing::instrument(level = "trace", skip(self)))]
    /// Get the handler for a given method and path
    pub fn get_handler(&self, method: &Method, path: &str) -> HandlerWrapper {
        let entry = self.get(method);

        if let Some(node) = entry {
            let route = Route::new(path);
            let handler = node.match_path(&route);

            if let Some(handler) = handler {
                debug!("Handler found for {} {}", method, path);
                handler.into()
            } else {
                debug!("No handler found for {} {}", method, path);
                (Self::not_found_handler(), BTreeMap::new()).into()
            }
        } else {
            debug!("No handler found for any {} method", method);
            (Self::not_found_handler(), BTreeMap::new()).into()
        }
    }

    /// internal method to get the not found handler
    fn not_found_handler() -> HandlerFn {
        HandlerFn::wrap(not_found)
    }

    /// Extends the handlers with a new set of handlers
    pub fn extend(&mut self, other: Handlers) {
        for (method, other_node) in other.0 {
            if let Some(node) = self.0.get_mut(&method) {
                node.extend(other_node);
            } else {
                self.0.insert(method, other_node);
            }
        }
    }
}

impl Deref for Handlers {
    type Target = HashMap<Method, RouteNode>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Handlers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

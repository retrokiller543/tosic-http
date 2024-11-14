mod not_found;
pub(crate) mod wrapper;

use crate::body::BoxBody;
use crate::error::Error;
use crate::handlers::not_found::not_found;
use crate::handlers::wrapper::HandlerWrapper;
use crate::route::{wrap_handler_fn, HandlerFn, Route, RouteNode};
use crate::traits::from_request::FromRequest;
use crate::traits::handler::Handler;
use crate::traits::responder::Responder;
use http::Method;
use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tracing::debug;

#[derive(Default, Debug, Clone)]
pub struct Handlers(pub HashMap<Method, RouteNode>);

impl Handlers {
    pub fn new() -> Self {
        Handlers(HashMap::new())
    }

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

    fn not_found_handler() -> Arc<HandlerFn> {
        wrap_handler_fn(Arc::new(not_found))
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

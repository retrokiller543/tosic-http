//! Routes and handlers are stored in a tree structure.

use crate::body::BoxBody;
use crate::error::Error;
use crate::request::{HttpPayload, HttpRequest};
use crate::response::HttpResponse;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::Service;
//use crate::handlers::HandlerType;
use super::{PathSegment, Route};
use crate::traits::from_request::FromRequest;
use crate::traits::handler::Handler;
use crate::traits::responder::Responder;

#[derive(Clone)]
/// [`HandlerFn`] is a wrapper around a handler function so that it can be used as a [`Service`].
pub struct HandlerFn(Arc<HandlerInner>);

impl HandlerFn {
    /// Create a new handler function from a handler
    pub(crate) fn wrap<Args>(handler: impl Handler<Args>) -> HandlerFn
    where
        Args: FromRequest + Send + 'static,
        Args::Future: Future + Send + 'static,
        Error: From<Args::Error>,
    {
        Self(wrap_handler_fn(Arc::new(handler)))
    }
}

impl Deref for HandlerFn {
    type Target = Arc<HandlerInner>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HandlerFn {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The inner type of [`HandlerFn`].
pub(crate) type HandlerInner = dyn Fn(
        HttpRequest,
        &mut HttpPayload,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
    + Send
    + Sync;

impl Service<(HttpRequest, HttpPayload)> for HandlerFn {
    type Response = HttpResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, mut req: (HttpRequest, HttpPayload)) -> Self::Future {
        self.0(req.0, &mut req.1)
    }
}

unsafe impl Send for HttpPayload {}
unsafe impl Send for BoxBody {}

/// Wrap a handler function so that it can be used as a [`Service`].
pub(crate) fn wrap_handler_fn<Args>(handler: Arc<impl Handler<Args>>) -> Arc<HandlerInner>
where
    Args: FromRequest + Send + 'static,
    Args::Future: Future + Send + 'static,
    Error: From<Args::Error>,
{
    Arc::new(
        #[inline]
        move |req: HttpRequest,
              payload: &mut HttpPayload|
              -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>> {
            let handler = handler.clone();
            let req = req;

            let mut payload = payload.clone();

            Box::pin(async move {
                let args = Args::from_request(&req, &mut payload).await?;
                let res = handler.call(args).await;

                Ok(res.respond_to(&req))
            })
        },
    ) as Arc<HandlerInner>
}

#[derive(Clone)]
/// A tree of routes.
///
/// It supports static, parameter, and wildcard routes.
///
/// ## Syntax
///
/// - `/` -> static route
/// - `/a/b` -> static route
/// - `/{param}/a` -> parameter as the first segment
/// - `/a/*/b` -> wildcard in the middle
/// - `/a/**` -> deep wildcard
pub struct RouteNode {
    static_children: HashMap<Cow<'static, str>, RouteNode>,
    parameter_child: Option<(Cow<'static, str>, Box<RouteNode>)>,
    wildcard_child: Option<Box<RouteNode>>,
    handler: Option<HandlerFn>,
}

impl Debug for RouteNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        enum DebugHandler {
            Some,
            None,
        }

        let mut binding = f.debug_struct("RouteNode");
        let f = binding
            .field("static_children", &self.static_children)
            .field("parameter_child", &self.parameter_child)
            .field("wildcard_child", &self.wildcard_child);

        let f = if self.handler.is_some() {
            f.field("handler", &DebugHandler::Some)
        } else {
            f.field("handler", &DebugHandler::None)
        };

        f.finish()
    }
}

impl Default for RouteNode {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteNode {
    /// Creates a new empty route node.
    pub fn new() -> Self {
        RouteNode {
            static_children: HashMap::new(),
            parameter_child: None,
            wildcard_child: None,
            handler: None,
        }
    }

    /// Extends the current route node with another route node.
    pub fn extend(&mut self, other: RouteNode) {
        for (key, other_child) in other.static_children {
            if let Some(child) = self.static_children.get_mut(&key) {
                child.extend(other_child);
            } else {
                self.static_children.insert(key, other_child);
            }
        }

        if let Some((other_param_name, other_child)) = other.parameter_child {
            if let Some((_param_name, child)) = &mut self.parameter_child {
                child.extend(*other_child);
            } else {
                self.parameter_child = Some((other_param_name, other_child));
            }
        }

        if let Some(other_child) = other.wildcard_child {
            if let Some(child) = &mut self.wildcard_child {
                child.extend(*other_child);
            } else {
                self.wildcard_child = Some(other_child);
            }
        }

        if other.handler.is_some() {
            self.handler = other.handler;
        }
    }

    /// Inserts a handler into the route node.
    pub fn insert<Args>(&mut self, route: &Route, handler: impl Handler<Args>)
    where
        Args: FromRequest + Send + 'static,
        Args::Future: Future + Send + 'static,
        Error: From<Args::Error>,
    {
        let handler_fn = HandlerFn::wrap(handler);

        self.insert_segments(route.segments(), handler_fn);
    }

    /// Inserts individual segments into the route node.
    fn insert_segments(&mut self, segments: &[PathSegment], handler: HandlerFn) {
        if segments.is_empty() {
            self.handler = Some(handler);
            return;
        }

        match &segments[0] {
            PathSegment::Static(segment) => {
                let child = self.static_children.entry(segment.clone()).or_default();
                child.insert_segments(&segments[1..], handler);
            }
            PathSegment::Parameter(param_name) => {
                if self.parameter_child.is_none() {
                    self.parameter_child = Some((param_name.clone(), Box::new(RouteNode::new())));
                }

                let (_, child_node) = self.parameter_child.as_mut().unwrap();
                child_node.insert_segments(&segments[1..], handler);
            }
            PathSegment::Wildcard => {
                if self.wildcard_child.is_none() {
                    self.wildcard_child = Some(Box::new(RouteNode::new()));
                }

                let child_node = self.wildcard_child.as_mut().unwrap();
                child_node.insert_segments(&segments[1..], handler);
            }
            PathSegment::WildcardDeep => {
                if self.wildcard_child.is_none() {
                    self.wildcard_child = Some(Box::new(RouteNode::new()));
                }

                let child_node = self.wildcard_child.as_mut().unwrap();
                // WildcardDeep means this node should handle all remaining segments
                child_node.handler = Some(handler.clone());
            }
        }
    }

    /// Matches a path against the route node.
    pub fn match_path(&self, route: &Route) -> Option<(HandlerFn, BTreeMap<String, String>)> {
        self.match_segments(route.segments())
    }

    /// Matches segments against the route node.
    pub fn match_segments(
        &self,
        segments: &[PathSegment],
    ) -> Option<(HandlerFn, BTreeMap<String, String>)> {
        if segments.is_empty() {
            return self
                .handler
                .clone()
                .map(|handler| (handler, BTreeMap::new()));
        }

        if let PathSegment::Static(segment) = &segments[0] {
            if let Some(child) = self.static_children.get(segment) {
                if let Some((handler, params)) = child.match_segments(&segments[1..]) {
                    return Some((handler, params));
                }
            }
        }

        if let Some((param_name, child)) = &self.parameter_child {
            if let Some((handler, mut params)) = child.match_segments(&segments[1..]) {
                if let PathSegment::Static(value) = &segments[0] {
                    params.insert(param_name.to_string(), value.to_string());
                }
                return Some((handler, params));
            }
        }

        if let Some(child) = &self.wildcard_child {
            return if let Some((handler, params)) = child.match_segments(&segments[1..]) {
                Some((handler, params))
            } else if let Some(_handler) = &child.handler {
                let remaining: Vec<_> = segments.to_vec();
                let mut params = BTreeMap::new();
                params.insert(
                    "wildcard_deep".to_string(),
                    remaining
                        .iter()
                        .map(|segment| format!("{}", segment))
                        .collect::<Vec<_>>()
                        .join("/"),
                );
                Some((child.handler.clone()?, params))
            } else {
                None
            };
        }

        None
    }
}

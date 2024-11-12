use crate::body::BoxBody;
use crate::error::Error;
use crate::request::{HttpPayload, HttpRequest};
use crate::response::HttpResponse;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
//use crate::handlers::HandlerType;
use super::{PathSegment, Route};
use crate::traits::from_request::FromRequest;
use crate::traits::handler::Handler;
use crate::traits::responder::Responder;

pub(crate) type HandlerFn = dyn Fn(
        HttpRequest,
        &mut HttpPayload,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
    + Send
    + Sync;

unsafe impl Send for HttpPayload {}
unsafe impl Send for BoxBody {}

pub(crate) fn wrap_handler_fn<H, Args>(handler: Arc<H>) -> Arc<HandlerFn>
where
    H: Handler<Args> + Send + Sync + 'static,
    Args: FromRequest + Send + 'static,
    Args::Future: Future + Send + 'static,
    H::Future: Future + Send + 'static,
    H::Output: Responder<Body = BoxBody> + 'static,
    Error: From<Args::Error>,
{
    Arc::new(
        move |req: HttpRequest,
              payload: &mut HttpPayload|
              -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>> {
            let handler = handler.clone();
            let req = req;

            let mut payload = payload.clone();

            Box::pin(async move {
                // Extract arguments from the request
                let args = Args::from_request(&req, &mut payload).await?;
                // Call the handler with the extracted arguments
                let res = handler.call(args).await;
                // Convert the handler's output into a response
                Ok(res.respond_to(&req))
            })
        },
    ) as Arc<HandlerFn>
}

#[derive(Clone)]
pub struct RouteNode {
    static_children: HashMap<Cow<'static, str>, RouteNode>,
    parameter_child: Option<(Cow<'static, str>, Box<RouteNode>)>,
    handler: Option<Arc<HandlerFn>>,
}

impl Debug for RouteNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouteNode")
            .field("static_children", &self.static_children)
            .field("parameter_child", &self.parameter_child)
            .finish()
    }
}

impl RouteNode {
    pub fn new() -> Self {
        RouteNode {
            static_children: HashMap::new(),
            parameter_child: None,
            handler: None,
        }
    }

    pub fn insert<H, Args>(&mut self, route: &Route, handler: H)
    where
        H: Handler<Args> + Send + Sync + 'static,
        Args: FromRequest + Send + 'static,
        Args::Future: Future + Send + 'static,
        H::Future: Future + Send + 'static,
        H::Output: Responder<Body = BoxBody> + 'static,
        Error: From<Args::Error>,
    {
        let handler_arc = Arc::new(handler);

        let handler_fn = wrap_handler_fn(handler_arc);

        self.insert_segments(route.segments(), handler_fn);
    }

    fn insert_segments(&mut self, segments: &[PathSegment], handler: Arc<HandlerFn>) {
        if segments.is_empty() {
            self.handler = Some(handler);
            return;
        }

        match &segments[0] {
            PathSegment::Static(segment) => {
                let child = self
                    .static_children
                    .entry(segment.clone())
                    .or_insert_with(RouteNode::new);
                child.insert_segments(&segments[1..], handler);
            }
            PathSegment::Parameter(param_name) => {
                if self.parameter_child.is_none() {
                    self.parameter_child = Some((param_name.clone(), Box::new(RouteNode::new())));
                }
                let (_, child_node) = self.parameter_child.as_mut().unwrap();
                child_node.insert_segments(&segments[1..], handler);
            }
            _ => {}
        }
    }

    pub fn match_path(&self, route: &Route) -> Option<(Arc<HandlerFn>, HashMap<String, String>)> {
        self.match_segments(route.segments())
    }

    fn match_segments(
        &self,
        segments: &[PathSegment],
    ) -> Option<(Arc<HandlerFn>, HashMap<String, String>)> {
        if segments.is_empty() {
            return self
                .handler
                .clone()
                .map(|handler| (handler, HashMap::new()));
        }

        // Try static children first
        if let PathSegment::Static(segment) = &segments[0] {
            if let Some(child) = self.static_children.get(segment) {
                if let Some((handler, params)) = child.match_segments(&segments[1..]) {
                    return Some((handler, params));
                }
            }
        }

        // Then try parameter child
        if let Some((param_name, child)) = &self.parameter_child {
            if let Some((handler, mut params)) = child.match_segments(&segments[1..]) {
                if let PathSegment::Static(value) = &segments[0] {
                    params.insert(param_name.to_string(), value.to_string());
                }
                return Some((handler, params));
            }
        }

        None
    }
}

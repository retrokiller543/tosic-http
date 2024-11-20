use std::borrow::Cow;
use std::future::Future;
use http::Method;
use paste::paste;
use crate::body::BoxBody;
use crate::error::Error;
use crate::handlers::Handlers;
use crate::prelude::{FromRequest, Handler, Responder};

pub struct RouteBuilder<'a> {
    path: Cow<'a, str>,
    handlers: Handlers,
}

macro_rules! route_method (
    {$($method:ident),+} => {
        $(
            paste! {
                pub fn $method<H, Args>(self, handler: H) -> Self
                where
                    H: Handler<Args> + Send + Sync + 'static,
                    Args: FromRequest + Send + 'static,
                    Args::Future: Future + Send + 'static,
                    H::Future: Future + Send + 'static,
                    H::Output: Responder<Body= BoxBody> + 'static,
                    Error: From<Args::Error>,
                {
                    self.insert_handler(Method::[<$method:snake:upper>], handler)
                }
            }
        )+
    };
);

macro_rules! route_function (
    {$($method:ident),+} => {
        $(
            paste! {
                pub fn $method<'a, H, Args>(path: &'a str, handler: H) -> RouteBuilder<'a>
                where
                    H: Handler<Args> + Send + Sync + 'static,
                    Args: FromRequest + Send + 'static,
                    Args::Future: Future + Send + 'static,
                    H::Future: Future + Send + 'static,
                    H::Output: Responder<Body= BoxBody> + 'static,
                    Error: From<Args::Error>,
                {
                    RouteBuilder::new(path).insert_handler(Method::[<$method:snake:upper>], handler)
                }
            }
        )+
    };
);

impl<'a> RouteBuilder<'a> {
    pub fn new(path: &'a str) -> Self {
        Self {
            path: path.into(),
            handlers: Handlers::new(),
        }
    }

    pub fn insert_handler<H, Args>(mut self, method: Method, handler: H) -> Self
    where
        H: Handler<Args> + Send + Sync + 'static,
        Args: FromRequest + Send + 'static,
        Args::Future: Future + Send + 'static,
        H::Future: Future + Send + 'static,
        H::Output: Responder<Body= BoxBody> + 'static,
        Error: From<Args::Error>,
    {
        if self.handlers.contains_key(&method) {
            panic!("You cant have more than one handler per method!")
        }

        self.handlers.insert(method, &*self.path, handler);

        self
    }

    pub(crate) fn handlers(self) -> Handlers {
        self.handlers
    }

    route_method! {get, post, put, delete, trace, options, head, connect, patch}
}

route_function! {get, post, put, delete, trace, options, head, connect, patch}
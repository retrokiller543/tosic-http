use crate::body::BoxBody;
use crate::error::Error;
use crate::handlers::Handlers;
use crate::server::HttpServer;
use crate::services::HttpService;
use crate::state::State;
use crate::traits::from_request::FromRequest;
use crate::traits::handler::Handler;
use crate::traits::responder::Responder;
use http::Method;
use std::fmt::Debug;
use std::future::Future;
use tokio::io;
use tokio::net::ToSocketAddrs;

#[derive(Default, Debug, Clone)]
pub struct HttpServerBuilder<T: ToSocketAddrs + Default + Clone> {
    addr: Option<T>,
    handlers: Handlers,
    app_state: State,
}

impl<T: ToSocketAddrs + Default + Clone + Debug> HttpServerBuilder<T> {
    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn app_state<S: Send + Sync + 'static>(self, state: S) -> Self {
        self.app_state.insert(state);

        self
    }

    pub fn service_method<H, Args>(mut self, method: Method, path: &str, handler: H) -> Self
    where
        H: Handler<Args> + Send + Sync + 'static,
        Args: FromRequest + Send + 'static,
        Args::Future: Future + Send + 'static,
        H::Future: Future + Send + 'static,
        H::Output: Responder<Body = BoxBody> + 'static,
        Error: From<Args::Error>,
    {
        self.handlers.insert(method, path, handler);
        self
    }

    pub fn service<H, Args>(mut self, handler: H) -> Self
    where
        H: HttpService<Args> + Handler<Args> + Send + Sync + 'static,
        Args: FromRequest + Send + 'static,
        Args::Future: Future + Send + 'static,
        H::Future: Future + Send + 'static,
        H::Output: Responder<Body = BoxBody> + 'static,
        Error: From<Args::Error>,
    {
        self.handlers.insert(H::METHOD, H::PATH, handler);
        self
    }

    pub fn addr(mut self, addr: T) -> Self {
        self.addr = Some(addr);
        self
    }

    pub async fn build(self) -> io::Result<HttpServer> {
        let addr = self.addr.unwrap_or_default();

        HttpServer::new(addr, self.handlers, self.app_state).await
    }
}

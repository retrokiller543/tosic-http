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

#[allow(unused_imports)]
use std::any::TypeId;

#[derive(Default, Debug, Clone)]
/// [`HttpServerBuilder`] is a builder for configuring and initializing an [`HttpServer`].
/// It allows for setting up the server address, adding services, and configuring shared state.
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

    /// Adds shared application state to be accessible in request handlers.
    ///
    /// State is stored in a [`HashMap`] and keyed based on the [`TypeId`] of the state object.
    ///
    /// # Arguments
    /// - `state`: A state object of type `S` that implements `Send + Sync + 'static`.
    ///
    /// # Returns
    /// The builder instance with the shared state added.
    ///
    /// # Examples
    /// ```
    /// # use tosic_http::prelude::HttpServer;
    /// struct MyState {
    ///     state: String
    /// }
    ///
    /// let builder = HttpServer::builder()
    ///     .with_state(MyState { state: "Hello, world!".to_string() })
    ///     .bind("127.0.0.1:8080");
    /// ```
    pub fn app_state<S: Send + Sync + 'static>(self, state: S) -> Self {
        self.app_state.insert(state);

        self
    }

    /// Adds a service handler to the server.
    ///
    /// # Arguments
    ///
    /// - `method`: The HTTP method for the service endpoint.
    /// - `path`: The path of the service endpoint.
    /// - `handler`: A handler implementing the [`Handler`] trait, defining a service endpoint.
    ///
    /// # Returns
    ///
    /// The builder instance with the handler added.
    ///
    /// # Examples
    /// ```
    /// # use tosic_http::prelude::{HttpServer, Responder, HttpResponse, BoxBody, Method};
    ///
    /// async fn basic_handler() -> impl Responder<Body = BoxBody> {
    ///     HttpResponse::new(200)
    /// }
    ///
    /// let builder = HttpServer::builder()
    ///     .service_method(Method::GET, "/", basic_handler)
    ///     .bind("127.0.0.1:8080");
    /// ```
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

    /// Adds a service handler to the server.
    ///
    /// # Arguments
    ///
    /// - `handler`: A handler implementing the [`Handler`] & [`HttpService`] traits.
    ///
    /// # Returns
    ///
    /// The builder instance with the handler added.
    ///
    /// # Examples
    /// ```
    /// # use tosic_http::prelude::{HttpServer, Responder, HttpResponse, BoxBody, Method, get};
    ///
    /// #[get("/")]
    /// async fn basic_handler() -> impl Responder<Body = BoxBody> {
    ///     HttpResponse::new(200)
    /// }
    ///
    /// let builder = HttpServer::builder()
    ///     .service(basic_handler)
    ///     .bind("127.0.0.1:8080");
    /// ```
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

    /// Sets the address the server will bind to.
    ///
    /// # Arguments
    ///
    /// - `addr`: The address for the server to bind, implementing [`ToSocketAddrs`].
    ///
    /// # Returns
    ///
    /// Returns the builder instance with the binding address set.
    ///
    /// # Examples
    /// ```
    /// # use tosic_http::prelude::HttpServer;
    /// let builder = HttpServer::builder()
    ///     .bind("127.0.0.1:8080");
    /// ```
    pub fn bind(mut self, addr: T) -> Self {
        self.addr = Some(addr);
        self
    }

    /// Builds and initializes the [`HttpServer`] with the current configuration.
    ///
    /// # Errors
    /// Returns [`io::Error`] if there was an error initializing the server.
    ///
    /// # Examples
    /// ```
    /// # use tosic_http::prelude::HttpServer;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let server = HttpServer::builder()
    ///     .bind("127.0.0.1:8080")
    ///     .build()
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn build(self) -> io::Result<HttpServer> {
        let addr = self.addr.unwrap_or_default();

        HttpServer::new(addr, self.handlers, self.app_state).await
    }
}

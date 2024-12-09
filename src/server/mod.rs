//! Main entry point for the HTTP server.

use crate::body::message_body::MessageBody;
use crate::error::{Error, ServerError};
use crate::handlers::Handlers;
use crate::request::{HttpPayload, HttpRequest};
use crate::response::HttpResponse;
use crate::route::HandlerFn;
use crate::server::builder::HttpServerBuilder;
use crate::state::State;
use http::HeaderMap;
use std::fmt::Debug;
use tokio::io;
use tokio::io::BufReader;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::ToSocketAddrs;
use tower::layer::util::Identity;
use tower::{Layer, Service, ServiceBuilder, ServiceExt};
#[cfg(feature = "trace")]
use tracing::trace;
use tracing::{debug, error, info};

pub mod builder;
mod test;

/// Represents a running HTTP server.
///
/// To construct a server, use [`HttpServer::builder`] or the builder struct directly [`HttpServerBuilder`].
pub struct HttpServer<L>
where
    L: Layer<HandlerFn> + Clone + Send + 'static,
{
    listener: tokio::net::TcpListener,
    handlers: Handlers,
    app_state: State,
    service_builder: ServiceBuilder<L>,
}

impl HttpServer<Identity> {
    /// Returns a new [`HttpServerBuilder`] for configuring and building an [`HttpServer`].
    pub fn builder<T: ToSocketAddrs + Default + Debug + Clone>() -> HttpServerBuilder<T, Identity> {
        HttpServerBuilder::<T, Identity>::new()
    }
}

impl<L> HttpServer<L>
where
    L: Layer<HandlerFn> + Clone + Send + 'static,
    L::Service: Service<(HttpRequest, HttpPayload), Response = HttpResponse, Error = Error>
        + Send
        + 'static,
    <L::Service as Service<(HttpRequest, HttpPayload)>>::Future: Send + 'static,
{
    #[cfg_attr(
        feature = "trace",
        tracing::instrument(level = "trace", skip(service_builder))
    )]
    /// Create a new [`HttpServer`] instance and binds the server to the provided address.
    ///
    /// This meant to be called from [`HttpServerBuilder`] and not externally
    pub(crate) async fn new(
        addr: impl ToSocketAddrs + Debug,
        handlers: Handlers,
        app_state: State,
        service_builder: ServiceBuilder<L>,
    ) -> io::Result<Self> {
        let listener = tokio::net::TcpListener::bind(addr).await?;

        #[cfg(feature = "trace")]
        trace!("Server Bound to {}", listener.local_addr()?);

        Ok(Self {
            listener,
            handlers,
            app_state,
            service_builder,
        })
    }

    /// Starts the server and listens for incoming connections.
    pub async fn serve(self) -> Result<(), ServerError> {
        info!("Listening on {}", self.listener.local_addr()?);
        loop {
            match self.listener.accept().await {
                Ok((stream, socket)) => {
                    #[cfg(feature = "trace")]
                    trace!("Accepted connection from {}", socket);
                    self.accept_connection(stream, socket)?;
                }
                Err(err) => {
                    error!("Failed to accept connection: {}", err);
                    continue;
                }
            }
        }
    }

    /// Main entry point for an incoming connection.
    ///
    /// In this step we spawn a new thread and handle the connection inside it to not block the main thread.
    fn accept_connection(
        &self,
        stream: tokio::net::TcpStream,
        socket: std::net::SocketAddr,
    ) -> Result<(), ServerError> {
        let handlers = self.handlers.clone();
        let state = self.app_state.clone();
        let service_builder = self.service_builder.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::handle_connection(
                stream,
                #[cfg(feature = "trace")]
                socket,
                handlers,
                state,
                service_builder,
            )
            .await
            {
                error!("Error handling connection from {}: {:?}", socket, e);
            }
        });

        Ok(())
    }

    #[cfg_attr(feature = "trace", tracing::instrument(level = "trace", skip_all))]
    /// Handles an incoming connection by reading the request, processing it, and sending the response
    async fn handle_connection(
        stream: tokio::net::TcpStream,
        #[cfg(feature = "trace")] socket: std::net::SocketAddr,
        handlers: Handlers,
        state: State,
        service_builder: ServiceBuilder<L>,
    ) -> Result<(), ServerError> {
        #[cfg(feature = "trace")]
        trace!("Accepted connection from {}", socket);

        let mut reader = BufReader::new(stream);

        let request_buffer = match Self::read_request(&mut reader).await {
            Ok(buffer) => buffer,
            Err(e) => {
                error!("Failed to read request: {}", e);
                return Err(e);
            }
        };

        let (mut request, payload) = match HttpRequest::from_bytes(&request_buffer) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse request: {}", e);
                return Err(e);
            }
        };

        request.data = state;

        #[cfg(feature = "trace")]
        trace!("Request: {:?}", request);

        let handler = handlers.get_handler(request.method(), request.uri().path());

        request.params_mut().extend(handler.1.clone());

        let mut service = service_builder.service(handler.handler());

        match service.ready().await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to construct service: {}", e);
                return Err(ServerError::ServiceConstructionFailed);
            }
        };

        let response = service.call((request, payload)).await.unwrap_or_else(|e| {
            error!("Failed to process request: {}", e);
            e.error_response()
        });

        Self::send_response(reader, response).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(level = "trace", skip(reader)))]
    /// Sends the response back to the client
    async fn send_response(
        reader: BufReader<tokio::net::TcpStream>,
        mut response: HttpResponse,
    ) -> Result<(), ServerError> {
        let content_length = response
            .body
            .clone()
            .try_into_bytes()
            .unwrap_or_default()
            .len() as u64;

        Self::insert_content_length(response.headers_mut(), content_length);

        response
            .headers_mut()
            .insert("Connection", "close".parse()?);

        let response_bytes = response.to_bytes()?;

        let mut stream = reader.into_inner();
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;

        Ok(())
    }

    fn insert_content_length(headers: &mut HeaderMap, content_length: u64) {
        headers.insert(
            "Content-Length",
            content_length.to_string().parse().unwrap(),
        );
    }

    #[cfg_attr(feature = "trace", tracing::instrument(level = "trace", skip(reader)))]
    /// Reads the request body and returns it as a vector of bytes
    async fn read_request(
        reader: &mut BufReader<tokio::net::TcpStream>,
    ) -> Result<Vec<u8>, ServerError> {
        let mut request_buffer = Vec::new();
        let mut headers_read = false;
        let mut content_length = 0;

        loop {
            let mut buf = [0; 1024];
            let n = reader.read(&mut buf).await?;

            if n == 0 {
                debug!("Connection closed by the client.");
                return Err(ServerError::ConnectionClosed);
            }

            request_buffer.extend_from_slice(&buf[..n]);

            if !headers_read {
                if let Some(headers_end) = Self::find_headers_end(&request_buffer) {
                    headers_read = true;

                    let headers = &request_buffer[..headers_end];
                    let headers_str = String::from_utf8_lossy(headers);

                    for line in headers_str.lines() {
                        if line.to_lowercase().starts_with("content-length:") {
                            if let Some(length_str) = line.split(':').nth(1) {
                                content_length = length_str.trim().parse::<usize>().unwrap_or(0);
                            }
                        }
                    }

                    let body_bytes_read = request_buffer.len() - headers_end;
                    if body_bytes_read >= content_length {
                        break;
                    }
                }
            } else {
                let total_bytes = request_buffer.len();
                let headers_end = Self::find_headers_end(&request_buffer).unwrap_or(0);
                let body_bytes_read = total_bytes - headers_end;

                if body_bytes_read >= content_length {
                    break;
                }
            }
        }

        Ok(request_buffer)
    }

    #[inline]
    /// Find the end of the request headers
    fn find_headers_end(buffer: &[u8]) -> Option<usize> {
        buffer
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|pos| pos + 4)
    }
}

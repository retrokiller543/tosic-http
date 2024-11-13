use std::sync::Arc;
use crate::error::{Error, ServerError};
use crate::handlers::Handlers;
use crate::request::{HttpPayload, HttpRequest};
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::io::BufReader;
use tokio::net::ToSocketAddrs;
use tracing::{debug, error, info, trace};
use crate::response::HttpResponse;
use crate::route::HandlerFn;

pub mod builder;
mod test;
pub struct HttpServer {
    listener: tokio::net::TcpListener,
    handlers: Handlers,
}

impl HttpServer {
    #[tracing::instrument(level = "trace", skip(handlers))]
    pub(crate) async fn new<T: ToSocketAddrs + std::fmt::Debug>(addr: T, handlers: Handlers) -> io::Result<Self> {
        let listener = tokio::net::TcpListener::bind(addr).await?;

        trace!("Server Bound to {}", listener.local_addr()?);

        Ok(Self { listener, handlers })
    }

    pub async fn serve(self) -> Result<(), ServerError> {
        info!("Listening on {}", self.listener.local_addr()?);
        loop {
            match self.listener.accept().await {
                Ok((stream, socket)) => {
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

    #[tracing::instrument(level = "trace", skip_all)]
    async fn handle_connection(
        stream: tokio::net::TcpStream,
        socket: std::net::SocketAddr,
        handlers: Handlers,
    ) -> Result<(), ServerError> {
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

        trace!("Request: {:?}", request);

        let handler = handlers.get_handler(request.method(), request.uri().path());

        request.params_mut().extend(handler.1.clone());

        let response = Self::process_request(handler.0, request, payload).await.unwrap_or_else(|e| {
            error!("Failed to process request: {}", e);
            e.error_response()
        });

        //let bytes = response.to_bytes()?;

        Self::send_response(reader, response).await
    }

    #[tracing::instrument(level = "trace", skip(reader))]
    async fn send_response(
        reader: BufReader<tokio::net::TcpStream>,
        response: HttpResponse,
    ) -> Result<(), ServerError> {
        let response_bytes = response.to_bytes()?;

        let mut stream = reader.into_inner();
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn process_request(handler: Arc<HandlerFn>, request: HttpRequest, mut payload: HttpPayload) -> Result<HttpResponse, Error> {
        let preprocessed_request = Self::pre_process_request(&request).await?;
        let response: HttpResponse = handler.call((preprocessed_request, &mut payload)).await?;
        let post_processed_response = Self::post_process_request(response).await?;

        Ok(post_processed_response)
    }

    /// Processes the request before passing it to the handler
    #[tracing::instrument(level = "trace", skip_all)]
    async fn pre_process_request(request: &HttpRequest) -> Result<HttpRequest, Error> {
        Ok(request.clone())
    }

    /// Processes the response we got from the handler before returning it to the client
    #[tracing::instrument(level = "trace", skip_all)]
    async fn post_process_request(response: HttpResponse) -> Result<HttpResponse, Error> {
        Ok(response)
    }

    #[allow(unused_variables)]
    fn accept_connection(
        &self,
        stream: tokio::net::TcpStream,
        socket: std::net::SocketAddr,
    ) -> Result<(), ServerError> {
        let handlers = self.handlers.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::handle_connection(stream, socket, handlers).await {
                error!("Error handling connection from {}: {:?}", socket, e);
            }
        });

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(reader))]
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
    fn find_headers_end(buffer: &[u8]) -> Option<usize> {
        buffer
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|pos| pos + 4)
    }
}

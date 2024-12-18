//! Compression middleware

use crate::body::message_body::MessageBody;
use crate::body::BoxBody;
use crate::error::ServerError;
use crate::prelude::{Error, HttpPayload, HttpRequest, HttpResponse};
use flate2::write::{DeflateEncoder, GzEncoder};
use flate2::Compression;
use std::future::Future;
use std::io::Write;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::warn;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Compression type
pub enum CompressionType {
    /// Gzip compression
    Gzip,
    /// Deflate compression
    Deflate,
}

#[derive(Clone, Debug)]
/// The compression layer to be used
pub struct CompressionLayer;

impl Default for CompressionLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl CompressionLayer {
    /// Create a new compression layer
    pub fn new() -> Self {
        Self
    }
}

impl<S: Clone> Layer<S> for CompressionLayer {
    type Service = CompressionMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        CompressionMiddleware { inner: service }
    }
}

#[derive(Clone, Debug)]
/// Compression middleware
pub struct CompressionMiddleware<S: Clone> {
    inner: S,
}

impl<S> Service<(HttpRequest, HttpPayload)> for CompressionMiddleware<S>
where
    S: Service<(HttpRequest, HttpPayload), Response = HttpResponse, Error = Error>
        + Clone
        + Send
        + Sync
        + 'static,
    S::Future: Send + 'static,
{
    type Response = HttpResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: (HttpRequest, HttpPayload)) -> Self::Future {
        let mut inner = self.inner.clone();
        let (request, payload) = req;

        let accept_encoding = request.headers().get("Accept-Encoding").cloned();

        Box::pin(async move {
            let mut response = inner.call((request, payload)).await?;

            let supported_encodings = vec![CompressionType::Gzip, CompressionType::Deflate];

            if let Some(encoding_header) = accept_encoding {
                if let Ok(encoding_str) = encoding_header.to_str() {
                    let client_encodings = parse_accept_encoding(encoding_str);

                    if let Some(best_encoding) =
                        negotiate_encoding(&client_encodings, &supported_encodings)
                    {
                        response = compress_response(response, best_encoding).await?;

                        let encoding_value = match best_encoding {
                            CompressionType::Gzip => "gzip",
                            CompressionType::Deflate => "deflate",
                        };
                        response
                            .headers_mut()
                            .insert("Content-Encoding", encoding_value.parse().unwrap());
                    } else {
                        warn!("No common encoding found between client and server");
                    }
                }
            }

            response
                .headers_mut()
                .insert("Vary", "Accept-Encoding".parse().unwrap());

            Ok(response)
        })
    }
}

/// Helper function to compress the response
async fn compress_response(
    mut response: HttpResponse,
    compression_type: CompressionType,
) -> Result<HttpResponse, ServerError> {
    // Read the body
    let body_bytes = response
        .body
        .clone()
        .try_into_bytes()
        .expect("Unable to read body");

    let compressed_body = match compression_type {
        CompressionType::Gzip => {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&body_bytes)?;
            encoder.finish()?
        }
        CompressionType::Deflate => {
            let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&body_bytes)?;
            encoder.finish()?
        }
    };

    let body = BoxBody::new(compressed_body);

    response = response.set_body(body);

    Ok(response)
}

/// Helper function to parse the Accept-Encoding header
fn parse_accept_encoding(header_value: &str) -> Vec<(CompressionType, f32)> {
    let mut encodings = Vec::new();

    for part in header_value.split(',') {
        let part = part.trim();
        let mut tokens = part.split(';');

        if let Some(encoding_str) = tokens.next() {
            let quality = tokens
                .find_map(|token| {
                    if token.trim().starts_with("q=") {
                        token.trim()[2..].parse::<f32>().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(1.0);

            let encoding = match encoding_str {
                "gzip" => Some(CompressionType::Gzip),
                "deflate" => Some(CompressionType::Deflate),
                "*" => Some(CompressionType::Gzip),
                _ => None,
            };

            if let Some(enc) = encoding {
                encodings.push((enc, quality));
            }
        }
    }

    encodings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    encodings
}

/// Helper function to negotiate the best encoding
fn negotiate_encoding(
    client_encodings: &[(CompressionType, f32)],
    server_encodings: &[CompressionType],
) -> Option<CompressionType> {
    for (encoding, _) in client_encodings {
        if server_encodings.contains(encoding) {
            return Some(*encoding);
        }
    }
    None
}

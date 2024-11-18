#![feature(impl_trait_in_assoc_type)]

use fake::faker::{filesystem::en::Semver, internet::en::MACAddress};
use fake::{Dummy, Fake, Faker};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use thiserror::Error;
use tokio::io;
use tosic_http::error::Error;
use tosic_http::prelude::{
    get, CompressionLayer, HttpPayload, HttpRequest, HttpResponse, HttpServer, Method,
};
use tosic_http::server::builder::HttpServerBuilder;
use tower::layer::util::Identity;
use tower::timeout::TimeoutLayer;
use tower::{Layer, Service};
use tracing::dispatcher::SetGlobalDefaultError;
use tracing::{error, info};

mod logger;

#[derive(Clone)]
pub struct LoggingMiddleware<S> {
    inner: S,
}

impl<S> Service<(HttpRequest, HttpPayload)> for LoggingMiddleware<S>
where
    S: Service<(HttpRequest, HttpPayload), Response = HttpResponse, Error = Error>
        + Send
        + Sync
        + 'static,
    S::Future: Send + 'static,
{
    type Response = HttpResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Ensure the inner service is ready to accept a request
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: (HttpRequest, HttpPayload)) -> Self::Future {
        //let mut inner = self.inner.clone();
        let method = req.0.method().clone();
        let uri = req.0.uri().clone();

        info!("Incoming request: {} {}", method, uri);

        let fut = self.inner.call(req);

        Box::pin(async move {
            // Log the incoming request
            info!("Incoming request: {} {}", method, uri);

            // Call the inner service
            let response = fut.await;

            // Log the response or error
            match &response {
                Ok(res) => {
                    info!("Response: {:?}", res);
                }
                Err(err) => {
                    error!("Error handling request: {}", err);
                }
            }

            response
        })
    }
}

#[derive(Clone)]
pub struct LoggingLayer;

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        LoggingMiddleware { inner: service }
    }
}

#[derive(Debug, Error)]
enum HttpServerError {
    #[error(transparent)]
    Tracing(#[from] SetGlobalDefaultError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Serialize, Deserialize, Dummy)]
pub struct Device {
    #[dummy(faker = "0..1000")]
    pub id: i32,
    #[dummy(faker = "MACAddress()")]
    pub mac: String,
    #[dummy(faker = "Semver()")]
    pub firmware: String,
}

//#[get("/api/devices")]
async fn devices() -> HttpResponse {
    let devices: Vec<Device> = Faker.fake();

    HttpResponse::Ok().json(&devices)
}

#[tokio::main]
async fn main() -> Result<(), HttpServerError> {
    logger::init_tracing()?;

    let server = HttpServer::<Identity>::builder()
        .wrap(CompressionLayer)
        .wrap(LoggingLayer)
        .bind("0.0.0.0:4221")
        .service_method(Method::GET, "/api/devices", devices)
        .build()
        .await?;

    match server.serve().await {
        Ok(_) => (),
        Err(e) => panic!("Failed to serve: {}", e),
    }

    Ok(())
}

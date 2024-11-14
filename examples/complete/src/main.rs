#![feature(impl_trait_in_assoc_type)]

use tosic_http::prelude::Method;
use fake::{Dummy, Fake, Faker};
use fake::faker::{self, internet::en::MACAddress, filesystem::en::Semver};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io;
use tracing::dispatcher::SetGlobalDefaultError;
use tosic_http::prelude::{get, HttpResponse};
use tosic_http::server::builder::HttpServerBuilder;

mod logger;

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

#[get("/api/devices")]
async fn devices() -> HttpResponse {
    let devices: Vec<Device> = Faker.fake();

    HttpResponse::Ok().json(&devices)
}

#[tokio::main]
async fn main() -> Result<(), HttpServerError> {
    logger::init_tracing()?;

    let server = HttpServerBuilder::default()
        .bind("0.0.0.0:4221")
        .service(devices)
        .build()
        .await?;

    match server.serve().await {
        Ok(_) => (),
        Err(e) => panic!("Failed to serve: {}", e),
    }

    Ok(())
}
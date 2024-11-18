#![feature(impl_trait_in_assoc_type)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io;
use tosic_http::prelude::HttpResponse;
use tosic_http::server::builder::HttpServerBuilder;
use tosic_http_macro::get;
use tracing::dispatcher::SetGlobalDefaultError;

mod logger {
    use crate::HttpServerError;
    #[cfg(feature = "log-subscriber")]
    use tracing::level_filters::LevelFilter;
    #[cfg(feature = "log-subscriber")]
    use tracing_subscriber::fmt::format::FmtSpan;
    #[cfg(feature = "log-subscriber")]
    use tracing_subscriber::fmt::Layer as FmtLayer;
    #[cfg(feature = "log-subscriber")]
    use tracing_subscriber::layer::SubscriberExt;
    #[cfg(feature = "log-subscriber")]
    use tracing_subscriber::{EnvFilter, Layer, Registry};

    #[cfg(feature = "log-subscriber")]
    pub fn init_tracing() -> Result<(), HttpServerError> {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            #[cfg(not(debug_assertions))]
            let level = LevelFilter::INFO;

            #[cfg(debug_assertions)]
            let level = LevelFilter::DEBUG;

            EnvFilter::builder()
                .with_default_directive(level.into())
                .from_env_lossy()
        });
        let def_layer = FmtLayer::new()
            .with_line_number(true)
            .with_span_events(FmtSpan::CLOSE)
            .with_level(true)
            .with_target(true)
            .with_thread_names(true)
            .with_thread_ids(true)
            .compact()
            .with_filter(filter);

        let subscriber = Registry::default().with(def_layer);

        tracing::subscriber::set_global_default(subscriber)?;

        Ok(())
    }

    #[cfg(all(feature = "console-subscriber", not(feature = "log-subscriber")))]
    pub fn init_tracing() -> Result<(), HttpServerError> {
        console_subscriber::init();

        Ok(())
    }
}

#[derive(Debug, Error)]
enum HttpServerError {
    #[error(transparent)]
    Tracing(#[from] SetGlobalDefaultError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: i32,
    pub mac: String,
    pub firmware: String,
}

#[get("/api/devices")]
async fn devices() -> HttpResponse {
    let devices = [
        Device {
            id: 1,
            mac: String::from("5F-33-CC-1F-43-82"),
            firmware: String::from("2.1.6"),
        },
        Device {
            id: 2,
            mac: String::from("EF-2B-C4-F5-D6-34"),
            firmware: String::from("2.1.5"),
        },
        Device {
            id: 3,
            mac: String::from("62-46-13-B7-B3-A1"),
            firmware: String::from("3.0.0"),
        },
        Device {
            id: 4,
            mac: String::from("96-A8-DE-5B-77-14"),
            firmware: String::from("1.0.1"),
        },
        Device {
            id: 5,
            mac: String::from("7E-3B-62-A6-09-12"),
            firmware: String::from("3.5.6"),
        },
    ];

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

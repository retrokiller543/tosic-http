use http::Method;
use serde::Deserialize;
use thiserror::Error;
use tokio::io;
use tosic_http::body::BoxBody;
use tosic_http::extractors::json::Json;
use tosic_http::extractors::query::Query;
use tosic_http::server::builder::HttpServerBuilder;
use tosic_http::traits::responder::Responder;
use tracing::dispatcher::SetGlobalDefaultError;

#[derive(Debug, Error)]
enum HttpServerError {
    #[error(transparent)]
    Tracing(#[from] SetGlobalDefaultError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

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
            let level = LevelFilter::TRACE;

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

#[derive(Deserialize)]
struct Test {
    username: String,
    password: String,
}

async fn test_handler(query: Query<Test>, json: Json<Test>) -> impl Responder<Body = BoxBody> {
    let test = query.into_inner();
    let json = json.into_inner();

    assert_eq!(test.username, json.username);

    format!("Hello, {}!", test.username)
}

#[tokio::main]
async fn main() -> Result<(), HttpServerError> {
    logger::init_tracing()?;

    let server = HttpServerBuilder::default()
        .addr("0.0.0.0:4221")
        .service_method(Method::POST, "/", test_handler)
        .build()
        .await?;

    match server.serve().await {
        Ok(_) => (),
        Err(e) => panic!("Failed to serve: {}", e),
    }

    Ok(())
}

use crate::HttpServerError;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::Layer as FmtLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer, Registry};

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
        .with_thread_ids(true)
        .compact()
        .with_filter(filter);

    let subscriber = Registry::default().with(def_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

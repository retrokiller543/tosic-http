#![allow(dead_code)]
#![feature(impl_trait_in_assoc_type)]

use http::Method;
use serde::Deserialize;
use thiserror::Error;
use tokio::io;
use tosic_http::body::BoxBody;
use tosic_http::error::response_error::ResponseError;
use tosic_http::extractors::json::Json;
use tosic_http::extractors::path::Path as HttpPath;
use tosic_http::extractors::query::Query;
use tosic_http::response::HttpResponse;
use tosic_http::server::builder::HttpServerBuilder;
use tosic_http::traits::responder::Responder;
use tosic_http_macro::get;
use tracing::dispatcher::SetGlobalDefaultError;

#[derive(Debug, Error)]
enum HttpServerError {
    #[error(transparent)]
    Tracing(#[from] SetGlobalDefaultError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl ResponseError for HttpServerError {}

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

#[derive(Deserialize, Debug)]
struct TestTest {
    username: String,
    password: String,
}

#[get("/{id}/{name}")]
async fn not_working(
    path: HttpPath<(u8, String)>,
) -> Result<impl Responder<Body = BoxBody>, HttpServerError> {
    let (id, name) = path.into_inner();

    Ok(HttpResponse::new(200).body((id.to_string(), name)))
}

async fn test_handler(
    query: Option<Query<TestTest>>,
    json: Json<TestTest>,
) -> impl Responder<Body = BoxBody> {
    let json = json.into_inner();

    if let Some(query) = query {
        let test = query.into_inner();

        assert_eq!(test.username, json.username);
        assert_eq!(test.password, json.password);
    }

    format!("Hello, {}!", json.username)
}

#[get("/test")]
async fn test_fn() -> impl Responder<Body = BoxBody> {
    "hello testing world"
}

struct State {
    dir: String,
}

/*#[get("**")]
#[tracing::instrument]
async fn website(req: HttpRequest, data: Data<State>) -> impl Responder<Body = BoxBody> {
    const DEFAULT_URL: &str = "index.html";
    let base_path = data.0.dir.clone();

    let file = match req.params().get("wildcard_deep") {
        Some(path) => {
            let path = Path::new(path);

            if path.starts_with("static.files") {
                // append ../
                path.to_path_buf()
            } else {
                path.to_path_buf()
            }
        }
        None => Path::new(DEFAULT_URL).to_path_buf(),
    };

    let path = Path::new(&base_path).join(file);

    dbg!(&path);

    if path.exists() {
        let content_type = if let Some(ext) = path.extension() {
            match ext.to_str().unwrap() {
                "html" => "text/html",
                "css" => "text/css",
                _ => "application/octet-stream",
            }
        } else {
            "text/plain"
        };

        let mut file = match File::open(path).await {
            Ok(f) => f,
            Err(err) => panic!("Error reading file {:?}", err),
        };

        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)
            .await
            .expect("TODO: panic message");
        file.flush().await.expect("TODO: panic message flush");

        let body = BoxBody::new(buffer);

        let mut response = HttpResponse::new(200);

        response
            .headers_mut()
            .insert(http::header::CONTENT_TYPE, content_type.parse().unwrap());

        response.set_body(body)
    } else {
        HttpResponse::new(404)
    }
}*/

#[tokio::main]
async fn main() -> Result<(), HttpServerError> {
    logger::init_tracing()?;
    let state = State {
        dir: "/Users/emil/projects/tosic-http/target/doc".to_string(),
    };

    let server = HttpServerBuilder::default()
        .app_state(state)
        .addr("0.0.0.0:4221")
        .service_method(Method::POST, "/", test_handler)
        //.service_method(Method::GET, "/bad", not_working)
        .service(not_working)
        .service(test_fn)
        //.service(website)
        .build()
        .await?;

    match server.serve().await {
        Ok(_) => (),
        Err(e) => panic!("Failed to serve: {}", e),
    }

    Ok(())
}

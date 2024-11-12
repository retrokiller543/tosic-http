use http::Method;
use tokio::io;
use tosic_http::body::BoxBody;
use tosic_http::server::builder::HttpServerBuilder;
use tosic_http::traits::responder::Responder;

async fn test_handler() -> impl Responder<Body = BoxBody> {
    "test response"
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let server = HttpServerBuilder::default()
        .addr("0.0.0.0:4221")
        .service_method(Method::GET, "/", test_handler)
        .build()
        .await?;

    match server.serve().await {
        Ok(_) => (),
        Err(e) => panic!("Failed to serve: {}", e),
    }

    Ok(())
}

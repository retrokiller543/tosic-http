#![cfg(test)]

use crate::body::BoxBody;
use crate::request::HttpRequest;
use crate::server::builder::HttpServerBuilder;
use crate::services::HttpService;
use crate::traits::handler::Handler;
use crate::traits::responder::Responder;
use http::Method;
use std::future::Future;

async fn test_handler(req: HttpRequest) -> impl Responder<Body = BoxBody> {
    "test response"
}

struct TestStructHandler;

impl Handler<(HttpRequest,)> for TestStructHandler {
    type Output = impl Responder<Body = BoxBody>;
    type Future = impl Future<Output = Self::Output>;

    fn call(&self, args: (HttpRequest,)) -> Self::Future {
        async fn test_handler(req: HttpRequest) -> impl Responder<Body = BoxBody> {
            "test response"
        }

        test_handler(args.0)
    }
}

impl HttpService<(HttpRequest,)> for TestStructHandler {
    const PATH: &'static str = "/example";
}

#[tokio::test]
async fn test() {
    let server = HttpServerBuilder::default()
        .service_method(Method::GET, "", test_handler)
        .service(TestStructHandler)
        .addr("0.0.0.0:0")
        .build()
        .await;

    assert!(server.is_ok());

    let server = server.unwrap();
}

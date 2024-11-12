use std::future::Future;
use http::Method;
use crate::body::BoxBody;
use crate::error::Error;
use crate::traits::from_request::FromRequest;
use crate::traits::handler::Handler;
use crate::traits::responder::Responder;

pub trait HttpService<Args>: Handler<Args> {
    const METHOD: Method = Method::GET;
    const PATH: &'static str = "";
}

/*impl<Args, H> HttpService<Args> for H
where
    H: Handler<Args> + Send + Sync + 'static,
    Args: FromRequest + Send + 'static,
    Args::Future: Future + Send + 'static,
    H::Future: Future + Send + 'static,
    H::Output: Responder<Body= BoxBody> + 'static,
    Error: From<Args::Error>,
{}*/

use crate::traits::handler::Handler;
use http::Method;

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

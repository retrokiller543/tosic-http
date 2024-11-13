use crate::traits::handler::Handler;
use http::Method;

pub trait HttpService<Args>: Handler<Args> {
    const METHOD: Method = Method::GET;
    const PATH: &'static str = "";
}

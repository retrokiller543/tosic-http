//! [`HttpService`] trait

use crate::traits::handler::Handler;
use http::Method;

/// A trait for defining a service endpoint.
pub trait HttpService<Args>: Handler<Args> {
    /// The HTTP method for the service endpoint.
    const METHOD: Method = Method::GET;
    /// The path of the service endpoint.
    const PATH: &'static str = "";
}

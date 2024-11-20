//! Prelude re-exports of the most used traits and types from the crate.

pub use tosic_http_macro::*;

pub use crate::body::BoxBody;
pub use crate::error::{Error, ResponseError, ServerError};
pub use crate::extractors::*;
#[cfg(feature = "utils")]
pub use crate::futures::*;
pub use crate::middleware::compression::*;
pub use crate::request::*;
pub use crate::response::*;
pub use crate::server::HttpServer;
pub use crate::services::HttpService;
pub use crate::traits::*;
pub use http::HeaderMap;
pub use http::Method;

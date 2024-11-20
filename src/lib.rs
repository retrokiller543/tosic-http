#![doc = include_str!("../README.md")]
#![feature(tuple_trait)]
#![feature(associated_type_defaults)]
#![feature(fn_traits)]
#![feature(impl_trait_in_assoc_type)]
//#![deny(missing_docs)]

pub mod body;
pub mod error;
pub mod extractors;
#[cfg(feature = "utils")]
pub mod futures;
#[cfg(not(feature = "utils"))]
pub(crate) mod futures;
pub(crate) mod handlers;
mod middleware;
pub mod prelude;
pub mod request;
pub mod resource;
pub mod response;
pub(crate) mod route;
pub mod server;
pub mod services;
pub(crate) mod state;
pub mod traits;
pub(crate) mod utils;

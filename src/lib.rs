#![feature(tuple_trait)]
#![feature(associated_type_defaults)]
#![feature(fn_traits)]
#![feature(impl_trait_in_assoc_type)]
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod body;
pub mod error;
pub mod extractors;
pub mod futures;
pub mod handlers;
pub mod request;
pub mod response;
pub mod route;
pub mod server;
pub mod services;
mod state;
pub mod traits;
pub mod utils;

pub use tosic_http_macro::*;

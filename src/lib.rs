#![feature(tuple_trait)]
#![feature(associated_type_defaults)]
#![feature(fn_traits)]
#![feature(impl_trait_in_assoc_type)]
#![allow(dead_code)]
#![allow(unused_variables)]

mod body;
mod error;
mod extractors;
mod futures;
mod handlers;
mod request;
mod response;
mod route;
mod server;
mod services;
mod traits;
mod utils;

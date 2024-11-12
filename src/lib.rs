#![feature(tuple_trait)]
#![feature(associated_type_defaults)]
#![feature(fn_traits)]
#![feature(impl_trait_in_assoc_type)]
#![allow(dead_code)]
#![allow(unused_variables)]

mod body;
mod error;
mod futures;
mod handlers;
mod request;
mod response;
mod route;
mod server;
mod services;
mod traits;
mod utils;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

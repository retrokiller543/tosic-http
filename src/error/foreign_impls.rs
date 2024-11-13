use crate::error::response_error::ResponseError;
use std::char::ParseCharError;
use std::convert::Infallible;
use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;

macro_rules! impl_parse_error (
    {$ty:ty} => {
        impl ResponseError for $ty {}
    };
);

impl_parse_error! {Infallible}
impl_parse_error! {ParseIntError}
impl_parse_error! {ParseBoolError}
impl_parse_error! {ParseCharError}
impl_parse_error! {ParseFloatError}

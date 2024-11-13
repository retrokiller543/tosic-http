use crate::error::response_error::ResponseError;
use crate::extractors::ExtractionError;
use crate::futures::{err, ok, Ready};
use crate::request::{HttpPayload, HttpRequest};
use crate::traits::from_request::FromRequest;
use std::fmt::{Debug, Display};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path<V>(pub V);

impl<T> Path<T> {
    #[inline]
    pub(crate) fn new(value: T) -> Self {
        Path(value)
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Path<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Path<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<V: FromStr> FromRequest for Path<Vec<V>>
where
    V::Err: ResponseError + Display + Debug,
{
    type Error = ExtractionError;
    type Future = Ready<Result<Self, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut HttpPayload) -> Self::Future {
        let params = req.params();

        let mut parsed_params = Vec::new();

        for value in params.values() {
            let parsed: V = match value.clone().parse() {
                Ok(val) => val,
                Err(error) => return err(ExtractionError::Path(error.to_string())),
            };

            parsed_params.push(parsed);
        }

        ok(Path::new(parsed_params))
    }
}

impl<V: FromStr, const N: usize> FromRequest for Path<[V; N]>
where
    V::Err: ResponseError + Display + Debug,
{
    type Error = ExtractionError;
    type Future = Ready<Result<Self, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut HttpPayload) -> Self::Future {
        let params = req.params();
        if params.len() != N {
            return err(ExtractionError::InvalidLength);
        }

        let mut parsed_params = Vec::new();
        for value in params.values() {
            let parsed: V = match value.clone().parse() {
                Ok(val) => val,
                Err(error) => return err(ExtractionError::Path(error.to_string())),
            };
            parsed_params.push(parsed);
        }

        match parsed_params.try_into() {
            Ok(array) => ok(Path::new(array)),
            Err(_) => err(ExtractionError::InvalidLength),
        }
    }
}

macro_rules! impl_tuple (
    {$($T:ident)+} => {
        #[allow(non_snake_case)]
        impl<$($T: FromStr),+> FromRequest for Path<($($T,)+)>
        where
            $($T::Err: ResponseError + Display + Debug,)+
        {
            type Error = ExtractionError;
            type Future = Ready<Result<Self, Self::Error>>;

            #[inline]
            fn from_request(req: &HttpRequest, _: &mut HttpPayload) -> Self::Future {
                let params = req.params();
                let mut iter = params.values();

                $(let $T: $T = match iter.next() {
                    Some(val) => match val.clone().parse() {
                        Ok(v) => v,
                        Err(error) => return err(ExtractionError::Path(error.to_string())),
                    },
                    None => return err(ExtractionError::MissingPathField),
                };)+

                let result = ($($T,)+);

                ok(Path::new(result))
            }
        }
    };
);

impl_tuple! {A}
impl_tuple! {A A1}
impl_tuple! {A A1 A2}
impl_tuple! {A A1 A2 A3}
impl_tuple! {A A1 A2 A3 A4}
impl_tuple! {A A1 A2 A3 A4 A5}
impl_tuple! {A A1 A2 A3 A4 A5 A6}
impl_tuple! {A A1 A2 A3 A4 A5 A6 A7}
impl_tuple! {A A1 A2 A3 A4 A5 A6 A7 A8}

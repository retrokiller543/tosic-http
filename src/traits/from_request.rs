//! # FromRequest
//!
//! The `FromRequest` trait is used to define how to extract data from the request

#![allow(non_snake_case)]

use crate::body::message_body::MessageBody;
use crate::body::BoxBody;
use crate::error::Error;
use crate::futures::{ok, Ready};
use crate::request::{HttpPayload, HttpRequest};
use bytes::Bytes;
use pin_project_lite::pin_project;
use std::convert::Infallible;
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

#[diagnostic::on_unimplemented(
    message = "Make sure to implement `FromRequest` if you wish to use `{Self}` as an extractor",
    label = "Consider not calling it here if this was intended and wrapping it function that may be used here",
    note = "The FromRequest trait is implemented on all tuples up to size 26 that are filled with types that implement `FromRequest`",
    note = "If you have more than 26 arguments the trait will not be implemented and you will need to restructure your endpoint"
)]
/// # FromRequest
///
/// The `FromRequest` trait is used to define how to extract data from the request
///
/// Any type that implements `FromRequest` can be used as an extractor and will be extracted from the request automatically in a handler
///
pub trait FromRequest: Sized {
    /// The error that can happen when extracting data
    type Error: Into<Error>;
    /// The future that will be used to extract the data
    type Future: Future<Output = Result<Self, Self::Error>>;

    /// Extracts a value of type `Self` from the request. The request contains the request body at the moment but that might change in the future
    fn from_request(req: &HttpRequest, payload: &mut HttpPayload) -> Self::Future;

    /// Extracts a value of type `Self` from the request
    fn extract(req: &HttpRequest) -> Self::Future {
        Self::from_request(req, &mut HttpPayload::default())
    }
}

pin_project! {
    /// Future for results `FromRequest`
    pub struct FromRequestResFuture<Fut, E> {
        #[pin]
        fut: Fut,
        _phantom: PhantomData<E>,
    }
}

pin_project! {
    /// Future for option `FromRequest`
    pub struct FromRequestOptFuture<Fut> {
        #[pin]
        fut: Fut,
    }
}

pin_project! {
    #[project = ExtractProj]
    #[project_replace = ExtractReplaceProj]
    enum ExtractFuture<Fut, Res> {
        Future {
            #[pin]
            fut: Fut
        },
        Done {
            output: Res,
        },
        Empty
    }
}

macro_rules! impl_tuple_from_request {
        ($fut: ident; $($T: ident),*) => {
            /// FromRequest implementation for tuple
            #[allow(unused_parens)]
            impl<$($T: FromRequest + 'static),+> FromRequest for ($($T,)+)
            {
                type Error = Error;
                type Future = $fut<$($T),+>;

                fn from_request(req: &HttpRequest, payload: &mut HttpPayload) -> Self::Future {
                    $fut {
                        $(
                            $T: ExtractFuture::Future {
                                fut: $T::from_request(req, payload),
                            },
                        )+
                    }
                }
            }

            pin_project! {
                /// Future for tuple
                pub struct $fut<$($T: FromRequest),+> {
                    $(
                        #[pin]
                        $T: ExtractFuture<$T::Future, $T>,
                    )+
                }
            }

            impl<$($T: FromRequest),+> Future for $fut<$($T),+>
            {
                type Output = Result<($($T,)+), Error>;

                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    let mut this = self.project();

                    let mut ready = true;
                    $(
                        match this.$T.as_mut().project() {
                            ExtractProj::Future { fut } => match fut.poll(cx) {
                                Poll::Ready(Ok(output)) => {
                                    let _ = this.$T.as_mut().project_replace(ExtractFuture::Done { output });
                                },
                                Poll::Ready(Err(err)) => return Poll::Ready(Err(err.into())),
                                Poll::Pending => ready = false,
                            },
                            ExtractProj::Done { .. } => {},
                            ExtractProj::Empty => unreachable!("FromRequest polled after finished"),
                        }
                    )+

                    if ready {
                        Poll::Ready(Ok(
                            ($(
                                match this.$T.project_replace(ExtractFuture::Empty) {
                                    ExtractReplaceProj::Done { output } => output,
                                    _ => unreachable!("FromRequest polled after finished"),
                                },
                            )+)
                        ))
                    } else {
                        Poll::Pending
                    }
                }
            }
        };
    }

impl_tuple_from_request!(TupleFromRequestFuture1; A);
impl_tuple_from_request!(TupleFromRequestFuture2; A, B);
impl_tuple_from_request!(TupleFromRequestFuture3; A, B, C);
impl_tuple_from_request!(TupleFromRequestFuture4; A, B, C, D);
impl_tuple_from_request!(TupleFromRequestFuture5; A, B, C, D, E);
impl_tuple_from_request!(TupleFromRequestFuture6; A, B, C, D, E, F);
impl_tuple_from_request!(TupleFromRequestFuture7; A, B, C, D, E, F, G);
impl_tuple_from_request!(TupleFromRequestFuture8; A, B, C, D, E, F, G, H);
impl_tuple_from_request!(TupleFromRequestFuture9; A, B, C, D, E, F, G, H, I);
impl_tuple_from_request!(TupleFromRequestFuture10; A, B, C, D, E, F, G, H, I, J);
impl_tuple_from_request!(TupleFromRequestFuture11; A, B, C, D, E, F, G, H, I, J, K);
impl_tuple_from_request!(TupleFromRequestFuture12; A, B, C, D, E, F, G, H, I, J, K, L);
impl_tuple_from_request!(TupleFromRequestFuture13; A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_tuple_from_request!(TupleFromRequestFuture14; A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_tuple_from_request!(TupleFromRequestFuture15; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_tuple_from_request!(TupleFromRequestFuture16; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_tuple_from_request!(TupleFromRequestFuture17; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_tuple_from_request!(TupleFromRequestFuture18; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_tuple_from_request!(TupleFromRequestFuture19; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_tuple_from_request!(TupleFromRequestFuture20; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_tuple_from_request!(TupleFromRequestFuture21; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_tuple_from_request!(TupleFromRequestFuture22; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_tuple_from_request!(TupleFromRequestFuture23; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_tuple_from_request!(TupleFromRequestFuture24; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_tuple_from_request!(TupleFromRequestFuture25; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_tuple_from_request!(TupleFromRequestFuture26; A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

impl FromRequest for () {
    type Error = Infallible;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(_: &HttpRequest, _: &mut HttpPayload) -> Self::Future {
        ok(())
    }
}

impl FromRequest for Bytes {
    type Error = Infallible;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(_: &HttpRequest, payload: &mut HttpPayload) -> Self::Future {
        ok(<BoxBody as Clone>::clone(payload)
            .boxed()
            .try_into_bytes()
            .expect("Unable to read body"))
    }
}

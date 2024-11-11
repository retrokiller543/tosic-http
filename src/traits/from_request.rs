#![allow(non_snake_case)]

use crate::error::Error;
use crate::futures::{ok, Ready};
use crate::request::HttpRequest;
use pin_project_lite::pin_project;
use std::convert::Infallible;
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

pub trait FromRequest: Sized {
    type Error: Into<Error>;
    type Future: Future<Output = Result<Self, Self::Error>>;

    /// Extracts a value of type `Self` from the request. The request contains the request body at the moment but that might change in the future
    fn from_request(req: &HttpRequest) -> Self::Future;
}

pin_project! {
    pub struct FromRequestResFuture<Fut, E> {
        #[pin]
        fut: Fut,
        _phantom: PhantomData<E>,
    }
}

pin_project! {
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

                fn from_request(req: &HttpRequest) -> Self::Future {
                    $fut {
                        $(
                            $T: ExtractFuture::Future {
                                fut: $T::from_request(req)
                            },
                        )+
                    }
                }
            }

            pin_project! {
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

    fn from_request(_: &HttpRequest) -> Self::Future {
        ok(())
    }
}

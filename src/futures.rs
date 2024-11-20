#![allow(dead_code)]
//! Utility functions for creating futures

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Clone)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
/// A future that is immediately ready
pub struct Ready<T> {
    /// The value of the future
    pub(crate) val: Option<T>,
}

impl<T> Ready<T> {
    /// Unwraps the value from this immediately ready future.
    #[inline]
    /// This is a no-op if the value is already taken.
    pub fn into_inner(mut self) -> T {
        self.val.take().unwrap()
    }
}

impl<T> Unpin for Ready<T> {}

impl<T> Future for Ready<T> {
    type Output = T;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
        let val = self.val.take().expect("Ready polled after completion");
        Poll::Ready(val)
    }
}

#[inline]
/// Creates a future that is immediately ready
pub fn ok<T, E>(val: T) -> Ready<Result<T, E>> {
    Ready { val: Some(Ok(val)) }
}

#[inline]
/// Creates a future that is immediately ready
pub fn ready<T>(val: T) -> Ready<T> {
    Ready { val: Some(val) }
}

#[inline]
/// Creates a future that is immediately ready with an error
pub fn err<T, E>(err: E) -> Ready<Result<T, E>> {
    Ready {
        val: Some(Err(err)),
    }
}

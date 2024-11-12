use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Clone)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Ready<T> {
    pub(crate) val: Option<T>,
}

impl<T> Ready<T> {
    /// Unwraps the value from this immediately ready future.
    #[inline]
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
pub fn ok<T, E>(val: T) -> Ready<Result<T, E>> {
    Ready { val: Some(Ok(val)) }
}

#[inline]
pub fn ready<T>(val: T) -> Ready<T> {
    Ready { val: Some(val) }
}

#[inline]
pub fn err<T, E>(err: E) -> Ready<Result<T, E>> {
    Ready {
        val: Some(Err(err)),
    }
}

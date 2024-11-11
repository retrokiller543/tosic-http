use crate::body::message_body::MessageBody;
use crate::body::size::BodySize;
use bytes::Bytes;
use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct None;

impl None {
    #[inline]
    pub fn new() -> Self {
        None
    }
}

impl MessageBody for None {
    type Error = Infallible;

    #[inline]
    fn size(&self) -> BodySize {
        BodySize::None
    }

    #[inline]
    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        Poll::Ready(Option::None)
    }

    #[inline]
    fn try_into_bytes(self) -> Result<Bytes, Self> {
        Ok(Bytes::new())
    }
}

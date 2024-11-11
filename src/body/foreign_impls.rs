use crate::body::message_body::MessageBody;
use crate::body::size::BodySize;
use bytes::{Bytes, BytesMut};
use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt::{Debug, Formatter};
use std::mem;
use std::ops::DerefMut;
use std::pin::Pin;
use std::task::{Context, Poll};

impl<B> MessageBody for &mut B
where
    B: MessageBody + Unpin + ?Sized,
{
    type Error = B::Error;

    fn size(&self) -> BodySize {
        (**self).size()
    }

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        Pin::new(&mut **self).poll_next(cx)
    }
}

impl MessageBody for Infallible {
    type Error = Infallible;

    fn size(&self) -> BodySize {
        match *self {}
    }

    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        match *self {}
    }
}

impl MessageBody for () {
    type Error = Infallible;

    #[inline]
    fn size(&self) -> BodySize {
        BodySize::Sized(0)
    }

    #[inline]
    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        Poll::Ready(None)
    }

    #[inline]
    fn try_into_bytes(self) -> Result<Bytes, Self> {
        Ok(Bytes::new())
    }
}

impl<B> MessageBody for Box<B>
where
    B: MessageBody + Unpin + ?Sized,
{
    type Error = B::Error;

    #[inline]
    fn size(&self) -> BodySize {
        self.as_ref().size()
    }

    #[inline]
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        Pin::new(self.get_mut().as_mut()).poll_next(cx)
    }
}

impl<T, B> MessageBody for Pin<T>
where
    T: DerefMut<Target = B> + Unpin + Debug,
    B: MessageBody + ?Sized,
{
    type Error = B::Error;

    #[inline]
    fn size(&self) -> BodySize {
        self.as_ref().size()
    }

    #[inline]
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        self.get_mut().as_mut().poll_next(cx)
    }
}

impl MessageBody for &'static [u8] {
    type Error = Infallible;

    #[inline]
    fn size(&self) -> BodySize {
        BodySize::Sized(self.len() as u64)
    }

    #[inline]
    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        if self.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Ok(Bytes::from_static(mem::take(self.get_mut())))))
        }
    }

    #[inline]
    fn try_into_bytes(self) -> Result<Bytes, Self> {
        Ok(Bytes::from_static(self))
    }
}

impl MessageBody for Bytes {
    type Error = Infallible;

    #[inline]
    fn size(&self) -> BodySize {
        BodySize::Sized(self.len() as u64)
    }

    #[inline]
    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        if self.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Ok(mem::take(self.get_mut()))))
        }
    }

    #[inline]
    fn try_into_bytes(self) -> Result<Bytes, Self> {
        Ok(self)
    }
}

impl MessageBody for BytesMut {
    type Error = Infallible;

    #[inline]
    fn size(&self) -> BodySize {
        BodySize::Sized(self.len() as u64)
    }

    #[inline]
    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        if self.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Ok(mem::take(self.get_mut()).freeze())))
        }
    }

    #[inline]
    fn try_into_bytes(self) -> Result<Bytes, Self> {
        Ok(self.freeze())
    }
}

impl MessageBody for Vec<u8> {
    type Error = Infallible;

    #[inline]
    fn size(&self) -> BodySize {
        BodySize::Sized(self.len() as u64)
    }

    #[inline]
    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        if self.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Ok(mem::take(self.get_mut()).into())))
        }
    }

    #[inline]
    fn try_into_bytes(self) -> Result<Bytes, Self> {
        Ok(Bytes::from(self))
    }
}

impl MessageBody for Cow<'static, [u8]> {
    type Error = Infallible;

    #[inline]
    fn size(&self) -> BodySize {
        BodySize::Sized(self.len() as u64)
    }

    #[inline]
    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        if self.is_empty() {
            Poll::Ready(None)
        } else {
            let bytes = match mem::take(self.get_mut()) {
                Cow::Borrowed(b) => Bytes::from_static(b),
                Cow::Owned(b) => Bytes::from(b),
            };
            Poll::Ready(Some(Ok(bytes)))
        }
    }

    #[inline]
    fn try_into_bytes(self) -> Result<Bytes, Self> {
        match self {
            Cow::Borrowed(b) => Ok(Bytes::from_static(b)),
            Cow::Owned(b) => Ok(Bytes::from(b)),
        }
    }
}

impl MessageBody for &'static str {
    type Error = Infallible;

    #[inline]
    fn size(&self) -> BodySize {
        BodySize::Sized(self.len() as u64)
    }

    #[inline]
    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        if self.is_empty() {
            Poll::Ready(None)
        } else {
            let string = mem::take(self.get_mut());
            let bytes = Bytes::from_static(string.as_bytes());
            Poll::Ready(Some(Ok(bytes)))
        }
    }

    #[inline]
    fn try_into_bytes(self) -> Result<Bytes, Self> {
        Ok(Bytes::from_static(self.as_bytes()))
    }
}

impl MessageBody for String {
    type Error = Infallible;

    #[inline]
    fn size(&self) -> BodySize {
        BodySize::Sized(self.len() as u64)
    }

    #[inline]
    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        if self.is_empty() {
            Poll::Ready(None)
        } else {
            let string = mem::take(self.get_mut());
            Poll::Ready(Some(Ok(Bytes::from(string))))
        }
    }

    #[inline]
    fn try_into_bytes(self) -> Result<Bytes, Self> {
        Ok(Bytes::from(self))
    }
}

impl MessageBody for Cow<'static, str> {
    type Error = Infallible;

    #[inline]
    fn size(&self) -> BodySize {
        BodySize::Sized(self.len() as u64)
    }

    #[inline]
    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        if self.is_empty() {
            Poll::Ready(None)
        } else {
            let bytes = match mem::take(self.get_mut()) {
                Cow::Borrowed(s) => Bytes::from_static(s.as_bytes()),
                Cow::Owned(s) => Bytes::from(s.into_bytes()),
            };
            Poll::Ready(Some(Ok(bytes)))
        }
    }

    #[inline]
    fn try_into_bytes(self) -> Result<Bytes, Self> {
        match self {
            Cow::Borrowed(s) => Ok(Bytes::from_static(s.as_bytes())),
            Cow::Owned(s) => Ok(Bytes::from(s.into_bytes())),
        }
    }
}

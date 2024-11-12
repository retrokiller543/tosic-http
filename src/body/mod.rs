//! All credit goes to [actix-web](https://github.com/actix/actix-web) for almost all of this code.

pub mod foreign_impls;
pub mod message_body;
pub mod none;
pub mod size;

use crate::body::message_body::{MessageBody, MessageBodyMapErr};
use crate::body::size::BodySize;
use bytes::Bytes;
use std::error::Error;
use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Clone)]
pub struct BoxBody(BoxBodyInner);

enum BoxBodyInner {
    None(none::None),
    Bytes(Bytes),
    Stream(Pin<Box<dyn MessageBody<Error = Box<dyn Error>>>>),
}

impl Clone for BoxBodyInner {
    fn clone(&self) -> Self {
        match self {
            BoxBodyInner::None(none) => BoxBodyInner::None(*none),
            BoxBodyInner::Bytes(b) => BoxBodyInner::Bytes(b.clone()),
            BoxBodyInner::Stream(_) => BoxBodyInner::None(none::None::new()),
        }
    }
}

impl Debug for BoxBodyInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoxBodyInner::None(none) => f.debug_tuple("Empty").field(none).finish(),
            BoxBodyInner::Bytes(b) => f.debug_tuple("Bytes").field(b).finish(),
            BoxBodyInner::Stream(_) => f.debug_tuple("Stream").finish(),
        }
    }
}

impl BoxBody {
    /// Boxes body type, erasing type information.
    ///
    /// If the body type to wrap is unknown or generic it is better to use [`MessageBody::boxed`] to
    /// avoid double boxing.
    #[inline]
    pub fn new<B>(body: B) -> Self
    where
        B: MessageBody + Clone + 'static,
    {
        match body.size() {
            BodySize::None => Self(BoxBodyInner::None(none::None::new())),
            _ => match body.try_into_bytes() {
                Ok(bytes) => Self(BoxBodyInner::Bytes(bytes)),
                Err(body) => {
                    let body = MessageBodyMapErr::new(body, Into::into);
                    Self(BoxBodyInner::Stream(Box::pin(body)))
                }
            },
        }
    }

    /// Returns a mutable pinned reference to the inner message body type.
    #[inline]
    pub fn as_pin_mut(&mut self) -> Pin<&mut Self> {
        Pin::new(self)
    }
}

impl MessageBody for BoxBody {
    type Error = Box<dyn Error>;

    #[inline]
    fn size(&self) -> BodySize {
        match &self.0 {
            BoxBodyInner::None(none) => none.size(),
            BoxBodyInner::Bytes(bytes) => bytes.size(),
            BoxBodyInner::Stream(stream) => stream.size(),
        }
    }

    #[inline]
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        match &mut self.0 {
            BoxBodyInner::None(body) => Pin::new(body).poll_next(cx).map_err(|err| match err {}),
            BoxBodyInner::Bytes(body) => Pin::new(body).poll_next(cx).map_err(|err| match err {}),
            BoxBodyInner::Stream(body) => Pin::new(body).poll_next(cx),
        }
    }

    #[inline]
    fn try_into_bytes(self) -> Result<Bytes, Self> {
        match self.0 {
            BoxBodyInner::None(body) => Ok(body.try_into_bytes().unwrap()),
            BoxBodyInner::Bytes(body) => Ok(body.try_into_bytes().unwrap()),
            _ => Err(self),
        }
    }

    #[inline]
    fn boxed(self) -> BoxBody {
        self
    }
}

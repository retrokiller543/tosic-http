use crate::body::size::BodySize;
use crate::body::BoxBody;
use bytes::Bytes;
use futures::ready;
use pin_project_lite::pin_project;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::task::{Context, Poll};

#[diagnostic::on_unimplemented(
    message = "The trait `MessageBody` is not implemented on `{Self}`",
    label = "Convert it to `Bytes` or `BytesMut` or any `String` type",
    note = "if you find that `MessageBody` should be implemented on `{Self}`, then please submit an issue and it might be added in the future"
)]
pub trait MessageBody: Debug {
    type Error: Into<Box<dyn std::error::Error>>;

    fn size(&self) -> BodySize;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>>;

    #[inline]
    fn try_into_bytes(self) -> Result<Bytes, Self>
    where
        Self: Sized,
    {
        Err(self)
    }

    #[inline]
    fn boxed(self) -> BoxBody
    where
        Self: Sized + Clone + 'static,
    {
        BoxBody::new(self)
    }
}

pin_project! {
    #[derive(Clone)]
    pub(crate) struct MessageBodyMapErr<B, F> {
        #[pin]
        body: B,
        mapper: Option<F>,
    }
}

impl<B, F, E> MessageBodyMapErr<B, F>
where
    B: MessageBody,
    F: FnOnce(B::Error) -> E,
{
    pub(crate) fn new(body: B, mapper: F) -> Self {
        Self {
            body,
            mapper: Some(mapper),
        }
    }
}

impl<B, F, E> Debug for MessageBodyMapErr<B, F>
where
    B: MessageBody,
    E: Into<Box<dyn std::error::Error>>,
    F: FnOnce(B::Error) -> E,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageBodyMapErr <...>").finish()
    }
}

impl<B, F, E> MessageBody for MessageBodyMapErr<B, F>
where
    B: MessageBody + Clone + Sized,
    F: FnOnce(B::Error) -> E,
    E: Into<Box<dyn std::error::Error>>,
{
    type Error = E;

    #[inline]
    fn size(&self) -> BodySize {
        self.body.size()
    }

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        let this = self.as_mut().project();

        match ready!(this.body.poll_next(cx)) {
            Some(Err(err)) => {
                let f = self.as_mut().project().mapper.take().unwrap();
                let mapped_err = (f)(err);
                Poll::Ready(Some(Err(mapped_err)))
            }
            Some(Ok(val)) => Poll::Ready(Some(Ok(val))),
            None => Poll::Ready(None),
        }
    }

    #[inline]
    fn try_into_bytes(self) -> Result<Bytes, Self> {
        let Self { body, mapper } = self;
        body.try_into_bytes().map_err(|body| Self { body, mapper })
    }
}

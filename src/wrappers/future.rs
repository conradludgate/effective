use std::{
    convert::Infallible,
    future::Future,
    future::IntoFuture,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{Async, EffectResult, Effective, Single};

/// Create an [`Effective`] that has no failures, a single value and is async
pub fn future<F: IntoFuture>(future: F) -> FromFuture<F::IntoFuture> {
    FromFuture {
        inner: future.into_future(),
    }
}

pin_project_lite::pin_project!(
    pub struct FromFuture<F> {
        #[pin]
        pub inner: F,
    }
);

impl<F: Future> Effective for FromFuture<F> {
    type Item = F::Output;
    type Failure = Infallible;
    type Produces = Single;
    type Async = Async;

    fn poll_effect(self: Pin<&mut Self>, cx: &mut Context<'_>) -> crate::EffectiveResult<Self> {
        match self.project().inner.poll(cx) {
            Poll::Ready(x) => EffectResult::Item(x),
            Poll::Pending => EffectResult::Pending(Async),
        }
    }
}

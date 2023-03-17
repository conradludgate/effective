use std::{
    convert::Infallible,
    future::IntoFuture,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Future;

use crate::{Async, EffectResult, Effective, Single};

pub fn future<F: IntoFuture>(future: F) -> FutureShim<F::IntoFuture> {
    FutureShim {
        inner: future.into_future(),
    }
}

pin_project_lite::pin_project!(
    pub struct FutureShim<F> {
        #[pin]
        pub inner: F,
    }
);

impl<F: Future> Effective for FutureShim<F> {
    type Item = F::Output;
    type Failure = Infallible;
    type Produces = Single;
    type Async = Async;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Failure, Self::Produces, Self::Async> {
        match self.project().inner.poll(cx) {
            Poll::Ready(x) => EffectResult::Item(x),
            Poll::Pending => EffectResult::Pending(Async),
        }
    }
}

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Future;

use crate::{EffectResult, Effective};

pub fn future<F>(future: F) -> FutureShim<F> {
    FutureShim { inner: future }
}

pin_project_lite::pin_project!(
    pub struct FutureShim<F> {
        #[pin]
        pub inner: F,
    }
);

impl<F: Future> Effective for FutureShim<F> {
    type Output = F::Output;
    type Residual = !;
    type Yields = !;
    type Awaits = ();

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Output, Self::Residual, Self::Yields, Self::Awaits> {
        match self.project().inner.poll(cx) {
            Poll::Ready(x) => EffectResult::Item(x),
            Poll::Pending => EffectResult::Pending(()),
        }
    }
}

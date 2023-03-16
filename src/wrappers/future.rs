use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Future;

use crate::{EffectResult, Effective, Okay};

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
    type Item = Okay<F::Output>;
    type Yields = !;
    type Awaits = ();

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Yields, Self::Awaits> {
        match self.project().inner.poll(cx) {
            Poll::Ready(x) => EffectResult::Item(Okay(x)),
            Poll::Pending => EffectResult::Pending(()),
        }
    }
}

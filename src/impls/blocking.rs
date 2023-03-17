//! Effect adaptors to subtract the 'async' effect.

use std::{
    future::poll_fn,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{Async, Blocking, EffectResult, Effective};

pub trait Executor {
    fn block_on<R>(&mut self, f: impl Future<Output = R>) -> R;
}

pin_project_lite::pin_project!(
    /// Produced by the [`block()`](super::EffectiveExt::block) method
    pub struct Block<E, R> {
        #[pin]
        pub(super) inner: E,
        pub(super) executor: R,
    }
);

impl<E, R> Effective for Block<E, R>
where
    E: Effective<Async = Async>,
    R: Executor,
{
    type Item = E::Item;
    type Failure = E::Failure;
    type Produces = E::Produces;
    type Async = Blocking;

    fn poll_effect(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Failure, Self::Produces, Self::Async> {
        let mut this = self.project();
        this.executor
            .block_on(poll_fn(|cx| match this.inner.as_mut().poll_effect(cx) {
                EffectResult::Item(x) => Poll::Ready(EffectResult::Item(x)),
                EffectResult::Failure(x) => Poll::Ready(EffectResult::Failure(x)),
                EffectResult::Done(x) => Poll::Ready(EffectResult::Done(x)),
                EffectResult::Pending(Async) => Poll::Pending,
            }))
    }
}

#[cfg(feature = "futures-executor")]
#[cfg_attr(docsrs, doc(cfg(feature = "futures-executor")))]
impl Executor for futures_executor::LocalPool {
    fn block_on<R>(&mut self, f: impl Future<Output = R>) -> R {
        self.run_until(f)
    }
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl Executor for tokio::runtime::Runtime {
    fn block_on<R>(&mut self, f: impl Future<Output = R>) -> R {
        tokio::runtime::Runtime::block_on(self, f)
    }
}

use std::{
    future::poll_fn,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{executor::LocalPool, Future};

use crate::{EffectResult, Effective, Async, Blocking};

#[derive(Default)]
pub struct FuturesExecutor {
    pool: LocalPool,
}

pub trait Executor {
    fn block_on<R>(&mut self, f: impl Future<Output = R>) -> R;
}

impl Executor for FuturesExecutor {
    fn block_on<R>(&mut self, f: impl Future<Output = R>) -> R {
        self.pool.run_until(f)
    }
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

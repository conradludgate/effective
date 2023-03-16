use std::{
    future::poll_fn,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{executor::LocalPool, Future};

use crate::{EffectResult, Effective};

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
    E: Effective<Awaits = ()>,
    R: Executor,
{
    type Output = E::Output;
    type Residual = E::Residual;
    type Yields = E::Yields;
    type Awaits = !;

    fn poll_effect(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> EffectResult<Self::Output, Self::Residual, Self::Yields, Self::Awaits> {
        let mut this = self.project();
        this.executor
            .block_on(poll_fn(|cx| match this.inner.as_mut().poll_effect(cx) {
                EffectResult::Item(x) => Poll::Ready(EffectResult::Item(x)),
                EffectResult::Failure(x) => Poll::Ready(EffectResult::Failure(x)),
                EffectResult::Done(x) => Poll::Ready(EffectResult::Done(x)),
                EffectResult::Pending(()) => Poll::Pending,
            }))
    }
}

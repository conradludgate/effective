//! Effect adaptors to subtract the 'iterable' effect

use std::{convert::Infallible, pin::Pin, task::Context};

use futures_util::task::noop_waker_ref;

use crate::{Async, Asynchronous, EffectResult, Effective, Fails, Multiple, Single};

pin_project_lite::pin_project!(
    /// Produced by the [`collect()`](super::EffectiveExt::collect) method
    pub struct Collect<E, C> {
        #[pin]
        pub(super) inner: E,
        pub(super) into: C,
    }
);

impl<E, C> Effective for Collect<E, C>
where
    E: Effective<Produces = Multiple>,
    C: Default + Extend<E::Item>,
{
    type Item = C;
    type Failure = E::Failure;
    type Produces = Single;
    type Async = E::Async;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Failure, Self::Produces, Self::Async> {
        let mut this = self.project();

        if !<Self::Async as Asynchronous>::IS_ASYNC && !<Self::Failure as Fails>::FALLIBLE {
            this.into.extend(CollectIterator { inner: this.inner });
            return EffectResult::Item(std::mem::take(this.into));
        }

        loop {
            match this.inner.as_mut().poll_effect(cx) {
                EffectResult::Item(x) => this.into.extend(Some(x)),
                EffectResult::Failure(x) => return EffectResult::Failure(x),
                EffectResult::Done(Multiple) => {
                    return EffectResult::Item(std::mem::take(this.into))
                }
                EffectResult::Pending(x) => return EffectResult::Pending(x),
            }
        }
    }
}

struct CollectIterator<'a, E> {
    inner: Pin<&'a mut E>,
}

impl<E> Iterator for CollectIterator<'_, E>
where
    E: Effective<Produces = Multiple>,
{
    type Item = E::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self
            .inner
            .as_mut()
            .poll_effect(&mut Context::from_waker(noop_waker_ref()))
        {
            EffectResult::Item(x) => Some(x),
            EffectResult::Failure(_) => unreachable!("FALLIBLE is false"),
            EffectResult::Done(Multiple) => None,
            EffectResult::Pending(_) => unreachable!("IS_ASYNC is false"),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<E, C> std::future::Future for Collect<E, C>
where
    E: Effective<Produces = Multiple, Async = Async, Failure = Infallible>,
    C: Default + Extend<E::Item>,
{
    type Output = C;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        match self.poll_effect(cx) {
            EffectResult::Item(value) => std::task::Poll::Ready(value),
            EffectResult::Failure(_) => unreachable!(),
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(_) => std::task::Poll::Pending,
        }
    }
}

use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;
use futures_util::task::noop_waker_ref;

use crate::{Async, Blocking, EffectResult, Effective, Multiple, Single, Failure};

pin_project_lite::pin_project!(
    /// `Shim` implements some of the well known third-party traits from [`Effective`].
    /// It can be constructed using [`EffectiveExt::shim`](crate::EffectiveExt::shim).
    pub struct Shim<T> {
        #[pin]
        pub inner: T,
    }
);

pin_project_lite::pin_project!(
    /// `TryShim` implements some of the well known third-party traits from [`Effective`].
    /// It can be constructed using [`EffectiveExt::try_shim`](crate::EffectiveExt::try_shim).
    pub struct TryShim<T> {
        #[pin]
        pub inner: T,
    }
);

impl<E> Iterator for Shim<E>
where
    E: Effective<Produces = Multiple, Async = Blocking, Failure = Infallible> + Unpin,
{
    type Item = E::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.inner).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => Some(x),
            EffectResult::Failure(_) => unreachable!(),
            EffectResult::Done(Multiple) => None,
            EffectResult::Pending(_) => unreachable!(),
        }
    }
}

impl<E> Future for Shim<E>
where
    E: Effective<Failure = Infallible, Produces = Single, Async = Async>,
{
    type Output = E::Item;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(x),
            EffectResult::Failure(_) => unreachable!(),
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(Async) => Poll::Pending,
        }
    }
}

impl<E> Stream for Shim<E>
where
    E: Effective<Produces = Multiple, Async = Async, Failure = Infallible>,
{
    type Item = E::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(Some(x)),
            EffectResult::Failure(_) => unreachable!(),
            EffectResult::Done(Multiple) => Poll::Ready(None),
            EffectResult::Pending(Async) => Poll::Pending,
        }
    }
}

impl<E, F> Iterator for TryShim<E>
where
    E: Effective<Produces = Multiple, Async = Blocking, Failure = Failure<F>> + Unpin,
{
    type Item = Result<E::Item, F>;

    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.inner).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => Some(Ok(x)),
            EffectResult::Failure(x) => Some(Err(x.0)),
            EffectResult::Done(Multiple) => None,
            EffectResult::Pending(_) => unreachable!(),
        }
    }
}

impl<E, F> Future for TryShim<E>
where
    E: Effective<Produces = Single, Async = Async, Failure = Failure<F>>,
{
    type Output = Result<E::Item, F>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(Ok(x)),
            EffectResult::Failure(x) => Poll::Ready(Err(x.0)),
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(Async) => Poll::Pending,
        }
    }
}

impl<E, F> Stream for TryShim<E>
where
    E: Effective<Produces = Multiple, Async = Async, Failure = Failure<F>>,
{
    type Item = Result<E::Item, F>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(Some(Ok(x))),
            EffectResult::Failure(x) => Poll::Ready(Some(Err(x.0))),
            EffectResult::Done(Multiple) => Poll::Ready(None),
            EffectResult::Pending(Async) => Poll::Pending,
        }
    }
}

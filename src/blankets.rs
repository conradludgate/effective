use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;
use futures_util::task::noop_waker_ref;

use crate::{Async, Blocking, EffectResult, Effective, Fallible, Multiple, ResultType, Single};

pin_project_lite::pin_project!(
    /// `Shim` implements some of the well known third-party traits from [`Effective`].
    /// It can be constructed using [`EffectiveExt::shim`](crate::EffectiveExt::shim).
    pub struct Shim<T> {
        #[pin]
        pub inner: T,
    }
);

impl<E> Iterator for Shim<E>
where
    E: Effective<Produces = Multiple, Async = Blocking> + Unpin,
{
    type Item = ResultType<E>;

    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.inner).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => Some(success::<E>(x)),
            EffectResult::Failure(x) => Some(failure::<E>(x)),
            EffectResult::Done(Multiple) => None,
            EffectResult::Pending(x) => match x {},
        }
    }
}

impl<E> Future for Shim<E>
where
    E: Effective<Produces = Single, Async = Async>,
{
    type Output = ResultType<E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(success::<E>(x)),
            EffectResult::Failure(x) => Poll::Ready(failure::<E>(x)),
            EffectResult::Done(x) => match x {},
            EffectResult::Pending(Async) => Poll::Pending,
        }
    }
}

impl<E> Stream for Shim<E>
where
    E: Effective<Produces = Multiple, Async = Async, Failure = Infallible>,
{
    type Item = ResultType<E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(Some(success::<E>(x))),
            EffectResult::Failure(x) => match x {},
            EffectResult::Done(Multiple) => Poll::Ready(None),
            EffectResult::Pending(Async) => Poll::Pending,
        }
    }
}

fn success<E: Effective>(x: E::Item) -> ResultType<E> {
    <E::Failure as Fallible>::success(x)
}

fn failure<E: Effective>(x: E::Failure) -> ResultType<E> {
    x.failure()
}

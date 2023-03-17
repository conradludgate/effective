use std::{
    convert::Infallible,
    future::Future,
    ops::ControlFlow,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use futures::{task::noop_waker_ref, Stream};

use crate::{
    Async, Blocking, EffectResult, Effective, Get, Multiple, Single, Try, TryAsyncIterator,
    TryFuture, TryGet, TryIterator,
};

pin_project_lite::pin_project!(
    /// Used for demonstrating how effective [`Effective`] is.
    pub struct Shim<T> {
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
impl<E> TryIterator for Shim<E>
where
    E: Effective<Produces = Multiple, Async = Blocking> + Unpin,
{
    type Output = E::Item;
    type Residual = E::Failure;

    fn try_next(&mut self) -> Option<ControlFlow<Self::Residual, Self::Output>> {
        match Pin::new(&mut self.inner).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => Some(ControlFlow::Continue(x)),
            EffectResult::Failure(x) => Some(ControlFlow::Break(x)),
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

impl<E> TryFuture for E
where
    E: Effective<Produces = Single, Async = Async>,
{
    type Output = E::Item;
    type Residual = E::Failure;

    fn try_poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<ControlFlow<Self::Residual, Self::Output>> {
        match self.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(ControlFlow::Continue(x)),
            EffectResult::Failure(x) => Poll::Ready(ControlFlow::Break(x)),
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

impl<E> TryAsyncIterator for E
where
    E: Effective<Produces = Multiple, Async = Async>,
{
    type Output = E::Item;
    type Residual = E::Failure;

    fn try_poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<ControlFlow<Self::Residual, Self::Output>>> {
        match self.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(Some(ControlFlow::Continue(x))),
            EffectResult::Failure(x) => Poll::Ready(Some(ControlFlow::Break(x))),
            EffectResult::Done(Multiple) => Poll::Ready(None),
            EffectResult::Pending(Async) => Poll::Pending,
        }
    }
}

impl<E> Get for E
where
    E: Effective<Produces = Single, Async = Blocking, Failure = Infallible>,
{
    type Output = E::Item;

    fn get(self) -> Self::Output {
        match pin!(self).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => x,
            EffectResult::Failure(_) => unreachable!(),
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(_) => unimplemented!(),
        }
    }
}

impl<E> TryGet for E
where
    E: Effective<Produces = Single, Async = Blocking>,
{
    type Continue = E::Item;
    type Break = E::Failure;

    fn try_get<R>(self) -> R
    where
        R: Try<Break = Self::Break, Continue = Self::Continue>,
    {
        match pin!(self).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => R::from_continue(x),
            EffectResult::Failure(x) => R::from_break(x),
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(_) => unimplemented!(),
        }
    }
}

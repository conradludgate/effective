use std::{
    async_iter::AsyncIterator,
    future::Future,
    ops::{ControlFlow, FromResidual, Try},
    pin::{pin, Pin},
    task::{Context, Poll},
};

use futures::{task::noop_waker_ref, Stream};

use crate::{EffectResult, Effective, Get, TryAsyncIterator, TryFuture, TryGet, TryIterator};

pin_project_lite::pin_project!(
    /// Used for demonstrating how effective [`Effective`] is.
    pub struct Shim<T> {
        #[pin]
        pub inner: T,
    }
);

impl<E> Iterator for Shim<E>
where
    E: Effective<Yields = (), Awaits = !, Residual = !> + Unpin,
{
    type Item = E::Output;

    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.inner).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => Some(x),
            EffectResult::Failure(_) => unreachable!(),
            EffectResult::Done(()) => None,
            EffectResult::Pending(_) => unreachable!(),
        }
    }
}
impl<E> TryIterator for Shim<E>
where
    E: Effective<Yields = (), Awaits = !> + Unpin,
{
    type Output = E::Output;
    type Residual = E::Residual;

    fn try_next(&mut self) -> Option<ControlFlow<Self::Residual, Self::Output>> {
        match Pin::new(&mut self.inner).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => Some(ControlFlow::Continue(x)),
            EffectResult::Failure(x) => Some(ControlFlow::Break(x)),
            EffectResult::Done(()) => None,
            EffectResult::Pending(_) => unreachable!(),
        }
    }
}

impl<E> Future for Shim<E>
where
    E: Effective<Residual = !, Yields = !, Awaits = ()>,
{
    type Output = E::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(x),
            EffectResult::Failure(_) => unreachable!(),
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(()) => Poll::Pending,
        }
    }
}

impl<E> TryFuture for E
where
    E: Effective<Yields = !, Awaits = ()>,
{
    type Output = E::Output;
    type Residual = E::Residual;

    fn try_poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<ControlFlow<Self::Residual, Self::Output>> {
        match self.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(ControlFlow::Continue(x)),
            EffectResult::Failure(x) => Poll::Ready(ControlFlow::Break(x)),
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(()) => Poll::Pending,
        }
    }
}

impl<E> AsyncIterator for Shim<E>
where
    E: Effective<Yields = (), Awaits = (), Residual = !>,
{
    type Item = E::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(Some(x)),
            EffectResult::Failure(_) => unreachable!(),
            EffectResult::Done(()) => Poll::Ready(None),
            EffectResult::Pending(()) => Poll::Pending,
        }
    }
}

impl<E> Stream for Shim<E>
where
    E: Effective<Yields = (), Awaits = (), Residual = !>,
{
    type Item = E::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(Some(x)),
            EffectResult::Failure(_) => unreachable!(),
            EffectResult::Done(()) => Poll::Ready(None),
            EffectResult::Pending(()) => Poll::Pending,
        }
    }
}

impl<E> TryAsyncIterator for E
where
    E: Effective<Yields = (), Awaits = ()>,
{
    type Output = E::Output;
    type Residual = E::Residual;

    fn try_poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<ControlFlow<Self::Residual, Self::Output>>> {
        match self.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(Some(ControlFlow::Continue(x))),
            EffectResult::Failure(x) => Poll::Ready(Some(ControlFlow::Break(x))),
            EffectResult::Done(()) => Poll::Ready(None),
            EffectResult::Pending(()) => Poll::Pending,
        }
    }
}

impl<E> Get for E
where
    E: Effective<Yields = !, Awaits = !, Residual = !>,
{
    type Output = E::Output;

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
    E: Effective<Yields = !, Awaits = !>,
{
    type Output = E::Output;
    type Residual = E::Residual;

    fn try_get<R>(self) -> R
    where
        R: FromResidual<Self::Residual> + Try<Output = Self::Output>,
    {
        match pin!(self).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => R::from_output(x),
            EffectResult::Failure(x) => R::from_residual(x),
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(_) => unimplemented!(),
        }
    }
}

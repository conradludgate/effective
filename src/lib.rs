#![feature(try_trait_v2, never_type, async_iterator)]

use std::{
    async_iter::AsyncIterator,
    future::Future,
    ops::{ControlFlow, FromResidual, Try},
    pin::{pin, Pin},
    task::{Context, Poll},
};

use futures::{task::noop_waker_ref, Stream};

/// A [`Try`] type that always continues and never breaks
pub struct Okay<T>(pub T);

impl<T> FromResidual<!> for Okay<T> {
    fn from_residual(_: !) -> Self {
        unreachable!()
    }
}

impl<T> Try for Okay<T> {
    type Output = T;
    type Residual = !;

    fn from_output(output: Self::Output) -> Self {
        Self(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        ControlFlow::Continue(self.0)
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for () {}
    impl Sealed for ! {}
}

pub trait Exists: private::Sealed {}
impl Exists for () {}
impl Exists for ! {}

pub enum EffectResult<Item, Yield, Await> {
    /// An item is ready
    Item(Item),
    /// No more items will be ready
    Done(Yield),
    /// No items are ready yet
    Pending(Await),
}

/// `Effective` encapsulates all possible effect types that
/// rust currently has. Fallability, Iterability and Awaitablilty.
pub trait Effective {
    /// Models how this effective type can fail.
    type Item: Try;
    /// Models whether this effective type can yield multiple values
    type Yields: Exists;
    /// Models whether this effective type can await
    type Awaits: Exists;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Yields, Self::Awaits>;
}

/// A useless trait with 0 possible effects.
pub trait Get {
    type Output;
    fn get(self) -> Self::Output;
}

/// [`Get`] + [`Try`]
pub trait TryGet {
    type Output: Try;
    fn try_get(self) -> Self::Output;
}

/// [`Try`] + [`Future`]
pub trait TryFuture {
    type Output: Try;
    fn try_poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
/// [`Try`] + [`AsyncIterator`]
pub trait TryAsyncIterator {
    type Output: Try;
    fn try_poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Output>>;
}

/// [`Try`] + [`Iterator`]
pub trait TryIterator {
    type Output: Try;
    fn try_next(&mut self) -> Option<Self::Output>;
}

pin_project_lite::pin_project!(
    /// Used for demonstrating how effective [`Effective`] is.
    pub struct Shim<T> {
        #[pin]
        pub inner: T,
    }
);

impl<E> Iterator for Shim<E>
where
    E: Effective<Yields = (), Awaits = !> + Unpin,
    E::Item: Try<Residual = !>,
{
    type Item = <E::Item as Try>::Output;

    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.inner).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => match x.branch() {
                ControlFlow::Continue(x) => Some(x),
                ControlFlow::Break(_) => unreachable!(),
            },
            EffectResult::Done(()) => None,
            EffectResult::Pending(_) => unreachable!(),
        }
    }
}
impl<E> TryIterator for Shim<E>
where
    E: Effective<Yields = (), Awaits = !> + Unpin,
{
    type Output = E::Item;

    fn try_next(&mut self) -> Option<Self::Output> {
        match Pin::new(&mut self.inner).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => Some(x),
            EffectResult::Done(()) => None,
            EffectResult::Pending(_) => unreachable!(),
        }
    }
}

impl<E> Future for Shim<E>
where
    E: Effective<Yields = !, Awaits = ()>,
    E::Item: Try<Residual = !>,
{
    type Output = <E::Item as Try>::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => match x.branch() {
                ControlFlow::Continue(x) => Poll::Ready(x),
                ControlFlow::Break(_) => unreachable!(),
            },
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(()) => Poll::Pending,
        }
    }
}

impl<E> TryFuture for E
where
    E: Effective<Yields = !, Awaits = ()>,
{
    type Output = E::Item;

    fn try_poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(x),
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(()) => Poll::Pending,
        }
    }
}

impl<E> AsyncIterator for Shim<E>
where
    E: Effective<Yields = (), Awaits = ()>,
    E::Item: Try<Residual = !>,
{
    type Item = <E::Item as Try>::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => match x.branch() {
                ControlFlow::Continue(x) => Poll::Ready(Some(x)),
                ControlFlow::Break(_) => unreachable!(),
            },
            EffectResult::Done(()) => Poll::Ready(None),
            EffectResult::Pending(()) => Poll::Pending,
        }
    }
}

impl<E> Stream for Shim<E>
where
    E: Effective<Yields = (), Awaits = ()>,
    E::Item: Try<Residual = !>,
{
    type Item = <E::Item as Try>::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => match x.branch() {
                ControlFlow::Continue(x) => Poll::Ready(Some(x)),
                ControlFlow::Break(_) => unreachable!(),
            },
            EffectResult::Done(()) => Poll::Ready(None),
            EffectResult::Pending(()) => Poll::Pending,
        }
    }
}

impl<E> TryAsyncIterator for E
where
    E: Effective<Yields = (), Awaits = ()>,
{
    type Output = E::Item;

    fn try_poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Output>> {
        match self.poll_effect(cx) {
            EffectResult::Item(x) => Poll::Ready(Some(x)),
            EffectResult::Done(()) => Poll::Ready(None),
            EffectResult::Pending(()) => Poll::Pending,
        }
    }
}

impl<E> Get for E
where
    E: Effective<Yields = !, Awaits = !>,
    E::Item: Try<Residual = !>,
{
    type Output = <E::Item as Try>::Output;

    fn get(self) -> Self::Output {
        match pin!(self).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => match x.branch() {
                ControlFlow::Continue(x) => x,
                ControlFlow::Break(_) => unreachable!(),
            },
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(_) => unimplemented!(),
        }
    }
}

impl<E> TryGet for E
where
    E: Effective<Yields = !, Awaits = !>,
{
    type Output = E::Item;

    fn try_get(self) -> Self::Output {
        match pin!(self).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => x,
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(_) => unimplemented!(),
        }
    }
}

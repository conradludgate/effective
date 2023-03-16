#![feature(try_trait_v2, never_type, async_iterator)]

use std::{
    ops::Try,
    pin::Pin,
    task::{Context, Poll},
};

mod blankets;
pub use blankets::{Okay, Shim};
pub mod impls;
pub mod wrappers;

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
/// Rust currently has. Fallability, Iterability and Awaitablilty.
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

/// [`Try`] + [`Future`](std::future::Future)
pub trait TryFuture {
    type Output: Try;
    fn try_poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
/// [`Try`] + [`AsyncIterator`](std::async_iter::AsyncIterator)
pub trait TryAsyncIterator {
    type Output: Try;
    fn try_poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Output>>;
}

/// [`Try`] + [`Iterator`]
pub trait TryIterator {
    type Output: Try;
    fn try_next(&mut self) -> Option<Self::Output>;
}

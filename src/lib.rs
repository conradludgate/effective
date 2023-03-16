#![feature(try_trait_v2, never_type, async_iterator)]

use std::{
    ops::{ControlFlow, FromResidual, Try},
    pin::Pin,
    task::{Context, Poll},
};

mod blankets;
pub use blankets::Shim;
pub mod impls;
pub mod wrappers;

mod private {
    use crate::Exists;

    pub trait Sealed {
        fn new() -> Self;
    }
    impl Sealed for () {
        fn new() -> Self {}
    }
    impl Sealed for ! {
        fn new() -> Self {
            unreachable!()
        }
    }

    pub trait MinExists<Lhs> {
        type Exists: Exists;
    }

    impl MinExists<()> for () {
        type Exists = ();
    }

    impl MinExists<()> for ! {
        type Exists = ();
    }

    impl MinExists<!> for () {
        type Exists = ();
    }

    impl MinExists<!> for ! {
        type Exists = !;
    }
}

pub trait Exists: private::Sealed {}
impl Exists for () {}
impl Exists for ! {}

pub enum EffectResult<Item, Failure, Yield, Await> {
    /// An item is ready
    Item(Item),
    /// A failure occured
    Failure(Failure),
    /// No more items will be ready
    Done(Yield),
    /// No items are ready yet
    Pending(Await),
}

/// `Effective` encapsulates all possible effect types that
/// Rust currently has. Fallability, Iterability and Awaitablilty.
pub trait Effective {
    /// What item does this effective type produce
    type Output;
    /// What non-success types can this effective produce
    type Residual;
    /// Models whether this effective type can yield multiple values
    type Yields: Exists;
    /// Models whether this effective type can await
    type Awaits: Exists;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Output, Self::Residual, Self::Yields, Self::Awaits>;
}

/// A useless trait with 0 possible effects.
pub trait Get {
    type Output;
    fn get(self) -> Self::Output;
}

/// [`Get`] + [`Try`]
pub trait TryGet {
    type Output;
    type Residual;
    fn try_get<R>(self) -> R
    where
        R: FromResidual<Self::Residual> + Try<Output = Self::Output>;
}

/// [`Try`] + [`Future`](std::future::Future)
pub trait TryFuture {
    type Output;
    type Residual;
    fn try_poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<ControlFlow<Self::Residual, Self::Output>>;
}
/// [`Try`] + [`AsyncIterator`](std::async_iter::AsyncIterator)
pub trait TryAsyncIterator {
    type Output;
    type Residual;
    fn try_poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<ControlFlow<Self::Residual, Self::Output>>>;
}

/// [`Try`] + [`Iterator`]
pub trait TryIterator {
    type Output;
    type Residual;
    fn try_next(&mut self) -> Option<ControlFlow<Self::Residual, Self::Output>>;
}

#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use std::{convert::Infallible, ops::ControlFlow, pin::Pin, task::Context};

mod blankets;
pub use blankets::Shim;
pub mod impls;
pub mod utils;
pub mod wrappers;

pub use impls::EffectiveExt;

mod private {
    use std::convert::Infallible;

    use crate::{Async, Blocking, Failure, Multiple, Single};

    pub trait SealedMarker {
        fn new() -> Self;
    }
    impl SealedMarker for Async {
        fn new() -> Self {
            Async
        }
    }
    impl SealedMarker for Blocking {
        fn new() -> Self {
            unreachable!()
        }
    }
    impl SealedMarker for Multiple {
        fn new() -> Self {
            Multiple
        }
    }
    impl SealedMarker for Single {
        fn new() -> Self {
            unreachable!()
        }
    }
    pub trait SealedFallible {}
    impl<T> SealedFallible for Failure<T> {}
    impl SealedFallible for Infallible {}
}
use private::{SealedFallible, SealedMarker};

/// Represents the act of awaiting a value
pub trait Asynchrony: SealedMarker {
    /// Does the value ever await?
    const IS_ASYNC: bool;
}

/// The value can wait
pub struct Async;
/// The value blocks
pub enum Blocking {}

impl Asynchrony for Async {
    const IS_ASYNC: bool = true;
}
impl Asynchrony for Blocking {
    const IS_ASYNC: bool = false;
}

/// Represents the act of producing many values
pub trait Iterable: SealedMarker + Sized {
    /// Does this actually produce multiple values?
    const MULTIPLE: bool;
}

/// Produces multiple values
pub struct Multiple;
/// Produces only a single value
pub enum Single {}

impl Iterable for Multiple {
    const MULTIPLE: bool = true;
}
impl Iterable for Single {
    const MULTIPLE: bool = false;
}

/// Represents the act of producing many values
pub trait Fallible: SealedFallible + From<Self::Failure> {
    type Failure;

    /// A suitable `Result` type to be paired with this failure
    type Result<T>;

    fn success<T>(t: T) -> Self::Result<T>;
    fn failure<T>(self) -> Self::Result<T>;

    /// Does this actually produce failure values?
    const FALLIBLE: bool;

    fn inner(self) -> Self::Failure;
}

/// Helper to get the result type of the effective.
///
/// It will be `E::Item` if the effective is infallible, otherwise it will be
/// `Result<E::Item, F>`
pub type ResultType<E> = <<E as Effective>::Failure as Fallible>::Result<<E as Effective>::Item>;

/// Represents a type that is fallible
pub struct Failure<T>(pub T);

impl<T: std::fmt::Debug> std::fmt::Debug for Failure<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl<T: std::fmt::Display> std::fmt::Display for Failure<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl<T: std::error::Error> std::error::Error for Failure<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl<T> From<T> for Failure<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<E> Fallible for Failure<E> {
    type Failure = E;

    type Result<T> = Result<T, E>;

    fn success<T>(t: T) -> Self::Result<T> {
        Ok(t)
    }
    fn failure<T>(self) -> Self::Result<T> {
        Err(self.0)
    }

    const FALLIBLE: bool = true;

    fn inner(self) -> Self::Failure {
        self.0
    }
}

impl Fallible for Infallible {
    type Failure = Infallible;

    type Result<T> = T;

    fn success<T>(t: T) -> Self::Result<T> {
        t
    }
    fn failure<T>(self) -> Self::Result<T> {
        unreachable!()
    }

    const FALLIBLE: bool = false;

    fn inner(self) -> Self::Failure {
        self
    }
}

/// The result of [`Effective::poll_effect`]
///
/// Represents the different effects that may be encountered
pub enum EffectResult<Item, Failure, Produces, Async> {
    /// An item is ready
    Item(Item),
    /// A failure occured
    Failure(Failure),
    /// No more items will be ready
    Done(Produces),
    /// No items are ready yet
    Pending(Async),
}

pub type EffectiveResult<E> = EffectResult<
    <E as Effective>::Item,
    <E as Effective>::Failure,
    <E as Effective>::Produces,
    <E as Effective>::Async,
>;

/// `Effective` encapsulates all possible effect types that
/// Rust currently has. Fallability, Iterability and Awaitablilty.
pub trait Effective {
    /// What item does this effective type produce
    type Item;
    /// What non-success types can this effective produce
    type Failure: Fallible;
    /// Models whether this effective type can produce multiple values
    type Produces: Iterable;
    /// Models whether this effective type can pause or will block
    type Async: Asynchrony;

    /// Attempt to pull out the next value of this effective.
    ///
    /// # Return value
    ///
    /// There are several possible return values, each indicating a distinct
    /// stream state:
    ///
    /// - `EffectResult::Pending(_)` means that this effectives's next value is not ready
    /// yet. Implementations will ensure that the current task will be notified
    /// when the next value may be ready.
    ///
    /// - `EffectResult::Item(val)` means that the effectives has successfully
    /// produced a value, `val`, and may produce further values on subsequent
    /// `poll_effect` calls. If this effective has `Produces = Single`, then
    /// `poll_effect` should not be invoked again.
    ///
    /// - `EffectResult::Done(_)` means that the effective has terminated, and
    /// `poll_effect` should not be invoked again.
    ///
    /// - `EffectResult::Failure(_)` means that there was a failure processing the next
    /// item in the effective. `poll_effect` should not be invoked again.
    ///
    /// # Panics
    ///
    /// Once a effective has finished (returned `EffectResult::Done(_)` from `poll_effect` or
    /// `EffectResult::Item(_)` when `Produces = Single`), calling its
    /// `poll_effect` method again may panic, block forever, or cause other kinds of
    /// problems; the `Effective` trait places no requirements on the effects of
    /// such a call. However, as the `poll_effect` method is not marked `unsafe`,
    /// Rust's usual rules apply: calls must never cause undefined behavior
    /// (memory corruption, incorrect use of `unsafe` functions, or the like),
    /// regardless of the stream's state.
    fn poll_effect(self: Pin<&mut Self>, cx: &mut Context<'_>) -> EffectiveResult<Self>;

    /// Returns the bounds on the remaining length of the stream.
    ///
    /// Specifically, `size_hint()` returns a tuple where the first element
    /// is the lower bound, and the second element is the upper bound.
    ///
    /// The second half of the tuple that is returned is an [`Option`]`<`[`usize`]`>`.
    /// A [`None`] here means that either there is no known upper bound, or the
    /// upper bound is larger than [`usize`].
    ///
    /// # Implementation notes
    ///
    /// It is not enforced that a effective implementation yields the declared
    /// number of elements. A buggy effective may yield less than the lower bound
    /// or more than the upper bound of elements.
    ///
    /// `size_hint()` is primarily intended to be used for optimizations such as
    /// reserving space for the elements of the effective, but must not be
    /// trusted to e.g., omit bounds checks in unsafe code. An incorrect
    /// implementation of `size_hint()` should not lead to memory safety
    /// violations.
    ///
    /// That said, the implementation should provide a correct estimation,
    /// because otherwise it would be a violation of the trait's protocol.
    ///
    /// The default implementation returns `(0, `[`None`]`)` which is correct for any
    /// effective, or `(1, Some(1))` if `Produces = Single`.
    fn size_hint(&self) -> (usize, Option<usize>) {
        if <Self::Produces as Iterable>::MULTIPLE {
            (0, None)
        } else {
            (1, Some(1))
        }
    }
}

/// A simpler stable imitation of [`Try`](std::ops::Try)
pub trait SimpleTry {
    /// The value which will be returned if execution should be stopped
    type Break;
    /// The value to proceed with if execution can continuie
    type Continue;
    /// Decide whether execution can continue
    fn branch(self) -> ControlFlow<Self::Break, Self::Continue>;
    /// When propragated from a break in the control flow, reconstruct the value
    fn from_break(b: Self::Break) -> Self;
    /// When propagated from a successfully completed control flow, reconstruct the value
    fn from_continue(c: Self::Continue) -> Self;
}

impl<T, E> SimpleTry for Result<T, E> {
    type Break = E;
    type Continue = T;

    fn branch(self) -> ControlFlow<Self::Break, Self::Continue> {
        match self {
            Ok(t) => ControlFlow::Continue(t),
            Err(e) => ControlFlow::Break(e),
        }
    }

    fn from_break(e: E) -> Self {
        Err(e)
    }
    fn from_continue(t: T) -> Self {
        Ok(t)
    }
}

impl<T> SimpleTry for Option<T> {
    type Break = ();
    type Continue = T;

    fn branch(self) -> ControlFlow<Self::Break, Self::Continue> {
        match self {
            Some(t) => ControlFlow::Continue(t),
            None => ControlFlow::Break(()),
        }
    }

    fn from_break(_: ()) -> Self {
        None
    }
    fn from_continue(t: T) -> Self {
        Some(t)
    }
}

impl<B, C> SimpleTry for ControlFlow<B, C> {
    type Break = B;
    type Continue = C;

    fn branch(self) -> Self {
        self
    }

    fn from_break(b: B) -> Self {
        ControlFlow::Break(b)
    }
    fn from_continue(c: C) -> Self {
        ControlFlow::Continue(c)
    }
}

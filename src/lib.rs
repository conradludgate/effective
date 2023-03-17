//! Effect handlers in Rust.
//! 
//! Inspired by <https://without.boats/blog/the-registers-of-rust/>, Rust can be considered to have
//! 3 well known effects:
//! * Asynchrony
//! * Iteration
//! * Fallibility
//! 
//! This is currently modelled in Rust using 3 different traits:
//! * [`Future`](std::future::Future)
//! * [`Iterator`]
//! * [`Try`](std::ops::Try)
//! 
//! The "Keyword Geenrics Initiative" have stirred up a little bit of controversy lately
//! by proposing some syntax that allows us to compose asynchrony with other traits in a generic
//! fashion. To put it another way, you can have an [`Iterator`] that is "maybe async" using syntax.
//! 
//! I find the idea interesting, but I think the syntax causes more confusion than it is useful.
//! 
//! I propose the [`Effective`] trait. As I previously mention, there are 3 effects. This
//! trait models all 3. Instead of _composing_ effects, you can _subtract_ effects.
//! 
//! For instance, [`Future`](std::future::Future) is [`Effective`] - [`Iterator`] - [`Try`](std::ops::Try):
//! 
//! ```
//! # use std::{future::Future, pin::Pin, task::{Context, Poll}};
//! # use std::convert::Infallible;
//! # struct Wrapper<E>(pub E);
//! # use effective::{Effective, Single, Async};
//! impl<E> Future for Wrapper<E>
//! where
//!     E: Effective<
//!         // Fallibility is turned off
//!         Failure = Infallible, 
//!         // Iteration is turned off
//!         Produces = Single, 
//!         // Asynchrony is kept on
//!         Async = Async
//!     >,
//! {
//!     type Output = E::Item;
//! 
//!     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//!         todo!()
//!     }
//! }
//! ```
//! 
//! ## Coloring problem
//! 
//! There's a well known blog post in the JS ecosystem called 
//! ["What color is your function?"](https://journal.stuffwithstuff.com/2015/02/01/what-color-is-your-function/).
//! It makes the claim that async functions in JS are a different color to 'non-async' functions.
//! I don't think the 'color' analogy really works in that case, since colors are known to be mixed.
//! 
//! However, it works perfectly with `Effective`.
//! 
//! ```ignore
//! // Red + Green + Blue
//! trait TryAsyncIterator = Effective<Failure = Error, Produces = Multiple, Async = Async>;
//! 
//! // Cyan (Blue + Green)
//! trait AsyncIterator = Effective<Failure = Infallible, Produces = Multiple, Async = Async>;
//! // Magenta (Red + Blue)
//! trait TryFuture = Effective<Failure = Error, Produces = Multiple, Async = Async>;
//! // Yellow (Green + Red)
//! trait TryIterator = Effective<Failure = Error, Produces = Multiple, Async = Blocking>;
//! 
//! // Red
//! trait Try = Effective<Failure = Error, Produces = Multiple, Async = Async>;
//! // Green
//! trait Iterator = Effective<Failure = Infallible, Produces = Multiple, Async = Blocking>;
//! // Blue
//! trait Future = Effective<Failure = Infallible, Produces = Single, Async = Async>;
//! 
//! // Black
//! trait FnOnce = Effective<Failure = Infallible, Produces = Single, Async = Blocking>;
//! ```
//! 
//! # Examples:
//! 
//! There are a lot of `map`-style functions. [`Iterator::map`],
//! [`Option::map`], [`FuturesExt::map`](futures::future::FutureExt::map),
//! [`TryStreamExt::map_ok`](futures::stream::TryStreamExt::map_ok).
//! 
//! They all do the same thing, map the success value to some other success value.
//! [`EffectiveExt`] also has a [`map`](EffectiveExt::map) method, but since [`Effective`]
//! can model all of those effects, it only needs a single method.
//! 
//! ## Try:
//!
//! ```
//! use effective::{impls::EffectiveExt, TryGet, wrappers};
//! 
//! // an effective with just fallible set
//! let e = wrappers::from_try(Some(42));
//!
//! let v: Option<i32> = e.map(|x| x + 1).try_get();
//! assert_eq!(v, Some(43));
//! ```
//!
//! ## Futures:
//!
//! ```
//! # async fn foo() {
//! use effective::{impls::EffectiveExt, wrappers};
//! 
//! // an effective with just async set
//! let e = wrappers::future(async { 0 });
//!
//! let v: i32 = e.map(|x| x + 1).shim().await;
//! # }
//! ```
//!
//! ## Iterators:
//!
//! ```
//! use effective::{impls::EffectiveExt, wrappers};
//! 
//! // an effective with just iterable set
//! let e = wrappers::iterator([1, 2, 3, 4]);
//!
//! let v: Vec<i32> = e.map(|x| x + 1).collect().get();
//! assert_eq!(v, [2, 3, 4, 5]);
//! ```
//!
//! ## Combined:
//!
//! ```
//! # async fn foo() {
//! use effective::{impls::EffectiveExt, wrappers};
//! 
//! async fn get_page(x: usize) -> Result<String, Box<dyn std::error::Error>> { 
//!     /* insert http request */
//! # Ok(x.to_string())
//! }
//! 
//! // an effective with just iterable set
//! let e = wrappers::iterator([1, 2, 3, 4])
//!     .map(get_page)
//!     // flatten in the async effect
//!     .flat_map(wrappers::future)
//!     // flatten in the fallible effect
//!     .flatten_try();
//!
//! let v: Vec<usize> = e.map(|x| x.len()).collect().unwrap().shim().await;
//! # }
//! ```
//! 
//! You'll also notice in that last example, we `map` with a fallible async function, and we
//! can use `flat_map`+`flatten_try` to embed the output into the effective directly.
//! 
//! This lets you **add** effects.
//! 
//! We can also **subtract** effects, we can see this in the `collect` method, but there are more.
//! 
//! ```ignore
//! // Effective<Failure = Infallible, Produces = Multiple, Async = Blocking>
//! let e = wrappers::iterator([1, 2, 3, 4]); 
//! 
//! // still the same effects for now...
//! let e = e.map(get_page); 
//! 
//! // Effective<Failure = Infallible, Produces = Multiple, Async = Async>
//! // We've flattened in in the 'async' effect.
//! let e = e.flat_map(wrappers::future); 
//! 
//! // Effective<Failure = Box<dyn std::error::Error>, Produces = Multiple, Async = Async>
//! // We've flattened in in the 'fallible' effect.
//! let e = e.flatten_try();
//! 
//! // Effective<Failure = Box<dyn std::error::Error>, Produces = Multiple, Async = Blocking>
//! // We've removed the async effect.
//! let e = e.block(FuturesExecutor::default());
//! 
//! // Effective<Failure = Box<dyn std::error::Error>, Produces = Single, Async = Blocking>
//! // We've removed the iterable effect.
//! let e = e.collect(); 
//! 
//! // Effective<Failure = Infallible, Produces = Single, Async = Blocking>
//! // We've removed the fallible effect.
//! let e = e.unwrap();
//! 
//! // no more effects, just a single value to get
//! let e: Vec<_> = e.get();
//! ```

use std::{
    ops::ControlFlow,
    pin::Pin,
    task::{Context, Poll},
};

mod blankets;
pub use blankets::Shim;
pub mod impls;
pub mod wrappers;

pub use impls::EffectiveExt;

mod private {
    use crate::{Async, Asynchronous, Blocking, Multiple, Produces, Single};

    pub trait Sealed {
        fn new() -> Self;
    }
    impl Sealed for Async {
        fn new() -> Self {
            Async
        }
    }
    impl Sealed for Blocking {
        fn new() -> Self {
            unreachable!()
        }
    }
    impl Sealed for Multiple {
        fn new() -> Self {
            Multiple
        }
    }
    impl Sealed for Single {
        fn new() -> Self {
            unreachable!()
        }
    }

    pub trait IsAsyncWith<Rhs>: Sized {
        type IsAsync: Asynchronous;
        fn into_max(self) -> Self::IsAsync {
            <Self::IsAsync>::new()
        }
        fn from_rhs(_: Rhs) -> Self::IsAsync {
            <Self::IsAsync>::new()
        }
    }

    impl IsAsyncWith<Async> for Async {
        type IsAsync = Async;
    }

    impl IsAsyncWith<Async> for Blocking {
        type IsAsync = Async;
    }

    impl IsAsyncWith<Blocking> for Async {
        type IsAsync = Async;
    }

    impl IsAsyncWith<Blocking> for Blocking {
        type IsAsync = Blocking;
    }

    pub trait ProducesMultipleWith<Rhs>: Sized {
        type Produces: Produces;
        fn into_max(self) -> Self::Produces {
            <Self::Produces>::new()
        }
        fn from_rhs(_: Rhs) -> Self::Produces {
            <Self::Produces>::new()
        }
    }

    impl ProducesMultipleWith<Multiple> for Multiple {
        type Produces = Multiple;
    }

    impl ProducesMultipleWith<Multiple> for Single {
        type Produces = Multiple;
    }

    impl ProducesMultipleWith<Single> for Multiple {
        type Produces = Multiple;
    }

    impl ProducesMultipleWith<Single> for Single {
        type Produces = Single;
    }
}

pub trait Asynchronous: private::Sealed {
    const IS_ASYNC: bool;
}

pub struct Async;
pub enum Blocking {}

impl Asynchronous for Async {
    const IS_ASYNC: bool = true;
}
impl Asynchronous for Blocking {
    const IS_ASYNC: bool = false;
}

pub trait Produces: private::Sealed {
    const MULTIPLE: bool;
}

pub struct Multiple;
pub enum Single {}

impl Produces for Multiple {
    const MULTIPLE: bool = true;
}
impl Produces for Single {
    const MULTIPLE: bool = false;
}

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

/// `Effective` encapsulates all possible effect types that
/// Rust currently has. Fallability, Iterability and Awaitablilty.
pub trait Effective {
    /// What item does this effective type produce
    type Item;
    /// What non-success types can this effective produce
    type Failure;
    /// Models whether this effective type can yield multiple values
    type Produces: Produces;
    /// Models whether this effective type can pause or will block
    type Async: Asynchronous;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Failure, Self::Produces, Self::Async>;
}

/// No effects. Just returns a value
pub trait Get {
    type Output;
    fn get(self) -> Self::Output;
}

/// [`Get`] + [`Try`]
pub trait TryGet {
    type Continue;
    type Break;
    fn try_get<R>(self) -> R
    where
        R: Try<Continue = Self::Continue, Break = Self::Break>;
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

pub trait Try {
    type Break;
    type Continue;
    fn branch(self) -> ControlFlow<Self::Break, Self::Continue>;
    fn from_break(b: Self::Break) -> Self;
    fn from_continue(c: Self::Continue) -> Self;
}

impl<T, E> Try for Result<T, E> {
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

impl<T> Try for Option<T> {
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

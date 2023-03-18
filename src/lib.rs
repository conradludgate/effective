#![cfg_attr(docsrs, feature(doc_cfg))]

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
//! The "Keyword Generics Initiative" have stirred up a little bit of controversy lately
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
//! [`Option::map`], [`FuturesExt::map`](futures_util::future::FutureExt::map),
//! [`TryStreamExt::map_ok`](futures_util::stream::TryStreamExt::map_ok).
//!
//! They all do the same thing, map the success value to some other success value.
//! [`EffectiveExt`] also has a [`map`](EffectiveExt::map) method, but since [`Effective`]
//! can model all of those effects, it only needs a single method.
//!
//! ## Try:
//!
//! ```
//! use effective::{impls::EffectiveExt, wrappers};
//!
//! // an effective with just fallible set
//! let e = wrappers::fallible(Some(42));
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
//!     .flatten_future()
//!     // flatten in the fallible effect
//!     .flatten_fallible();
//!
//! let v: Vec<usize> = e.map(|x| x.len()).collect().unwrap().shim().await;
//! # }
//! ```
//!
//! You'll also notice in that last example, we `map` with a fallible async function, and we
//! can use `flat_map`+`flatten_fallible` to embed the output into the effective directly.
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
//! let e = e.flatten_future();
//!
//! // Effective<Failure = Box<dyn std::error::Error>, Produces = Multiple, Async = Async>
//! // We've flattened in in the 'fallible' effect.
//! let e = e.flatten_fallible();
//!
//! let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
//!
//! // Effective<Failure = Box<dyn std::error::Error>, Produces = Multiple, Async = Blocking>
//! // We've removed the async effect.
//! let e = e.block_on(runtime);
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
//!
//! # North Star
//!
//! Obviously this library is quite complex to reason about. I think it is a good pairing to
//! keyword-generics.
//!
//! There should be a syntax to implement these concepts but I think the underlying trait is a good
//! abstraction.
//!
//! Similar to how:
//! * `async{}.await` models a [`Future`](std::future::Future),
//! * `try{}?` models a [`Try`](std::ops::Try),
//! * `for/yield` models an [`Iterator`]
//!
//! These syntax elements could be composed to make application level [`Effective`] implementations.
//!
//! ```ignore
//! async try get_page(after: Option<usize>) -> Result<Option<Page>, Error> { todo!() }
//!
//! // `async` works as normal, allows the `.await` syntax
//! // `gen` allows the `yield` syntax, return means `Done`
//! // `try` allows the `?` syntax.
//! async gen try fn get_pages() -> Result<Page, Error> {
//!     let Some(mut page) = get_page(None).await? else {
//!         // no first page, exit
//!         return;
//!     };
//!
//!     loop {
//!         let next_page = page.next_page;
//!
//!         // output the page (auto 'ok-wrapping')
//!         yield page;
//!
//!         let Some(p) = get_page(Some(next_page)).await? else {
//!             // no next page, exit
//!             return;
//!         };
//!
//!         page = p;
//!     }
//! }
//!
//! // This method is still `async` and `try`, but it removes the 'gen` keyword
//! // because internally we handle all the iterable effects.
//! async try fn save_pages() -> Result<(), Error> {
//!     // The for loop is designed to handle iterable effects only.
//!     // `try` and `await` here tell it to expect and propagate the
//!     // fallible and async effects.
//!     for try await page in get_pages() {
//!         page.save_to_disk()?
//!     }
//! }
//! ```
//!
//! With adaptors, it would look like
//!
//! ```ignore
//! let get_pages = wrappers::unfold(None, |next_page| {
//!     wrappers::future(get_page(next_page)).flatten_fallible().map(|page| {
//!         page.map(|| {
//!             let next_page = page.next_page;
//!             (page, Some(next_page))
//!         })
//!     })
//! });
//!
//! let save_pages = get_pages.for_each(|page| {
//!     wrappers::future(page.save_to_disk()).flatten_fallible()
//! });
//! ```

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
pub trait Asynchronous: SealedMarker {
    /// Does the value ever await?
    const IS_ASYNC: bool;
}

/// The value can wait
pub struct Async;
/// The value blocks
pub enum Blocking {}

impl Asynchronous for Async {
    const IS_ASYNC: bool = true;
}
impl Asynchronous for Blocking {
    const IS_ASYNC: bool = false;
}

/// Represents the act of producing many values
pub trait Produces: SealedMarker {
    /// Does this actually produce multiple values?
    const MULTIPLE: bool;
}

/// Produces multiple values
pub struct Multiple;
/// Produces only a single value
pub enum Single {}

impl Produces for Multiple {
    const MULTIPLE: bool = true;
}
impl Produces for Single {
    const MULTIPLE: bool = false;
}

/// Represents the act of producing many values
pub trait Fails: SealedFallible + From<Self::Failure> {
    type Failure;
    /// Does this actually produce failure values?
    const FALLIBLE: bool;

    fn inner(self) -> Self::Failure;
}

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

impl<T> Fails for Failure<T> {
    type Failure = T;
    const FALLIBLE: bool = true;

    fn inner(self) -> Self::Failure {
        self.0
    }
}
impl Fails for Infallible {
    type Failure = Infallible;
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

/// `Effective` encapsulates all possible effect types that
/// Rust currently has. Fallability, Iterability and Awaitablilty.
pub trait Effective {
    /// What item does this effective type produce
    type Item;
    /// What non-success types can this effective produce
    type Failure: Fails;
    /// Models whether this effective type can produce multiple values
    type Produces: Produces;
    /// Models whether this effective type can pause or will block
    type Async: Asynchronous;

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
    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Failure, Self::Produces, Self::Async>;

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
        if <Self::Produces as Produces>::MULTIPLE {
            (0, None)
        } else {
            (1, Some(1))
        }
    }
}

/// A simpler stable imitation of [`Try`](std::ops::Try)
pub trait Fallible {
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

impl<T, E> Fallible for Result<T, E> {
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

impl<T> Fallible for Option<T> {
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

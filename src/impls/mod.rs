//! Where common [`Effective`] adaptors live

use std::{convert::Infallible, future::Future, pin::pin, task::Context};

use futures_util::task::noop_waker_ref;

use crate::{
    blankets::TryShim,
    utils::{HasFailureWith, IsAsyncWith, ProducesMultipleWith},
    wrappers::{FromFallible, FromFuture, FromIterator},
    Async, Blocking, EffectResult, Effective, Failure, Fallible, Multiple, Shim, Single,
};

use self::blocking::Executor;

pub mod blocking;
pub mod collect;
pub mod flatten;
pub mod fold;
pub mod for_each;
pub mod map;
pub mod unwrap;

type FromTryFn<T> = fn(T) -> FromFallible<T>;
type FromIterFn<T> = fn(T) -> FromIterator<T>;
type FromFutFn<T> = fn(T) -> FromFuture<T>;

/// Common adaptors to [`Effective`].
pub trait EffectiveExt: Effective {
    /// Apply the function over the items, returning a new effective, Flattening the result
    /// into a single effective.
    fn flat_map<R, F>(self, f: F) -> flatten::Flatten<map::Map<Self, F>>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: Effective,
        Self::Produces: ProducesMultipleWith<<R as Effective>::Produces>,
        Self::Async: IsAsyncWith<<R as Effective>::Async>,
        Self::Failure: HasFailureWith<<R as Effective>::Failure>,
    {
        self.map(f).flatten()
    }

    /// If the `Item` of this effective is fallible, it pulls that flattens into the effective.
    ///
    /// # Note
    ///
    /// The effective must be currently infallible. You can use `e.map(fallible).flatten()`
    /// if you already have a failure case.
    fn flatten_fallible(self) -> flatten::Flatten<map::Map<Self, FromTryFn<Self::Item>>>
    where
        Self: Sized,
        Self: Effective<Failure = Infallible>,
        Self::Item: Fallible,
        Self::Produces: ProducesMultipleWith<Single>,
        Self::Async: IsAsyncWith<Blocking>,
    {
        self.map(crate::wrappers::fallible as _).flatten()
    }

    /// If the `Item` of this effective is a future, it pulls that future into the effective.
    fn flatten_future(self) -> flatten::Flatten<map::Map<Self, FromFutFn<Self::Item>>>
    where
        Self: Sized,
        Self::Item: Future,
        Self::Async: IsAsyncWith<Async>,
        Self::Produces: ProducesMultipleWith<Single>,
        Self::Failure: HasFailureWith<Infallible>,
    {
        self.map(crate::wrappers::future as _).flatten()
    }

    /// If the `Item` of this effective is an iterator, it pulls that iterator into the effective.
    fn flatten_iterator(self) -> flatten::Flatten<map::Map<Self, FromIterFn<Self::Item>>>
    where
        Self: Sized,
        Self::Item: Iterator,
        Self::Async: IsAsyncWith<Blocking>,
        Self::Produces: ProducesMultipleWith<Multiple>,
        Self::Failure: HasFailureWith<Infallible>,
    {
        self.map(crate::wrappers::iterator as _).flatten()
    }

    /// Map the items in the effective
    ///
    /// # Example
    ///
    /// ## Try:
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, wrappers};
    /// let e = wrappers::fallible(Some(42));
    ///
    /// let v: Option<i32> = e.map(|x| x + 1).try_get();
    /// assert_eq!(v, Some(43));
    /// ```
    ///
    /// ## Futures:
    ///
    /// ```
    /// # async fn foo() {
    /// use effective::{impls::EffectiveExt, wrappers};
    /// let e = wrappers::future(async { 0 });
    ///
    /// let v: i32 = e.map(|x| x + 1).shim().await;
    /// # }
    /// ```
    ///
    /// ## Iterators:
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, wrappers};
    /// let e = wrappers::iterator([1, 2, 3, 4]);
    ///
    /// let v: Vec<i32> = e.map(|x| x + 1).collect().get();
    /// assert_eq!(v, [2, 3, 4, 5]);
    /// ```
    fn map<R, F>(self, f: F) -> map::Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
    {
        map::Map {
            inner: self,
            map: f,
        }
    }

    /// If this effective item is itself an effective, flatten those items into a single effective.
    fn flatten(self) -> flatten::Flatten<Self>
    where
        Self: Sized,
        Self::Item: Effective,
        Self::Produces: ProducesMultipleWith<<Self::Item as Effective>::Produces>,
        Self::Async: IsAsyncWith<<Self::Item as Effective>::Async>,
        Self::Failure: HasFailureWith<<Self::Item as Effective>::Failure>,
    {
        flatten::Flatten {
            inner: Some(self),
            flatten: None,
        }
    }

    /// Collect the items from this iterator into a collection.
    ///
    /// Can be thought of as subtracting the 'iterable' effect.
    ///
    /// # Example
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, wrappers};
    /// let e = wrappers::iterator([1, 2, 3, 4]);
    ///
    /// let v: Vec<i32> = e.collect().get();
    /// ```
    fn collect<C>(self) -> collect::Collect<Self, C>
    where
        Self: Sized,
        Self: Effective<Produces = Multiple>,
        C: Default + Extend<Self::Item>,
    {
        collect::Collect {
            inner: self,
            into: Default::default(),
        }
    }

    /// Block on the async effective
    ///
    /// Can be thought of as subtracting the 'async' effect.
    ///
    /// # Example
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, wrappers};
    ///
    /// let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
    ///
    /// let e = wrappers::iterator([1, 2, 3, 4])
    ///     .map(|x| async move { x + 1 })
    ///     .flatten_future();
    ///
    /// let v: Vec<i32> = e.block_on(runtime).collect().get();
    /// assert_eq!(v, [2, 3, 4, 5]);
    /// ```
    fn block_on<R>(self, executor: R) -> blocking::Block<Self, R>
    where
        Self: Sized,
        Self: Effective<Async = Async>,
        R: Executor,
    {
        blocking::Block {
            inner: self,
            executor,
        }
    }

    /// Panic on the fallible effective
    ///
    /// Can be thought of as subtracting the 'fallable' effect.
    ///
    /// # Example
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, wrappers};
    ///
    /// let e = wrappers::from_fn_once(|| 1_i32)
    ///     .map(|x| x.checked_add(1))
    ///     .flatten_fallible();
    ///
    /// let v = e.unwrap().get();
    /// assert_eq!(v, 2);
    /// ```
    fn unwrap(self) -> unwrap::Unwrap<Self>
    where
        Self: Sized,
        Self::Failure: std::fmt::Debug,
    {
        unwrap::Unwrap { inner: self }
    }

    /// Extract the value if there are no more effects possible
    fn get(self) -> Self::Item
    where
        Self: Sized,
        Self: Effective<Produces = Single, Failure = Infallible, Async = Blocking>,
    {
        match pin!(self).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => x,
            EffectResult::Failure(_) => unreachable!(),
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(_) => unimplemented!(),
        }
    }

    /// Extract the value or failure
    fn try_get<R, F>(self) -> R
    where
        Self: Sized,
        Self: Effective<Produces = Single, Async = Blocking, Failure = Failure<F>>,
        R: Fallible<Continue = Self::Item, Break = F>,
    {
        match pin!(self).poll_effect(&mut Context::from_waker(noop_waker_ref())) {
            EffectResult::Item(x) => R::from_continue(x),
            EffectResult::Failure(x) => R::from_break(x.0),
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(_) => unimplemented!(),
        }
    }

    /// Return a [`shim`](Shim) that implements either [`Future`], [`Stream`](futures_core::stream::Stream) or [`Iterator`]
    fn shim(self) -> Shim<Self>
    where
        Self: Sized,
    {
        Shim { inner: self }
    }

    /// Return a [`shim`](Shim) that implements either [`TryFuture`](futures_core::TryFuture), [`TryStream`](futures_core::TryStream) or [`Iterator`]
    fn try_shim(self) -> TryShim<Self>
    where
        Self: Sized,
    {
        TryShim { inner: self }
    }

    /// High level fold function. Takes all the items in the effective and applies the `func` to it,
    /// with a running accumulator. Returns the final accumulator value.
    ///
    /// `F` must return a new effective, this must only have a single value but can be async or fallible.
    ///
    /// # Example
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, wrappers};
    ///
    /// let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
    ///
    /// // async for maximum efficiency 😎
    /// async fn multiply(a: i32, b: i32) -> Option<i32> {
    ///     a.checked_mul(b)
    /// }
    ///
    /// let e = wrappers::iterator([2, 3, 4, 5]);
    ///
    /// let v: Option<i32> = e.fold(1, |acc, item| {
    ///     wrappers::future(multiply(acc, item)).flatten_fallible()
    /// }).block_on(runtime).try_get();
    ///
    /// assert_eq!(v, Some(120));
    /// ```
    fn fold<F, B, C>(self, init: B, func: F) -> fold::Fold<Self, F, B, C>
    where
        Self: Sized,
        Self: Effective<Produces = Multiple>,
        F: FnMut(B, Self::Item) -> C,
        C: Effective<Item = B, Produces = Single>,
    {
        fold::Fold {
            inner: self,
            func,
            state: fold::State::Acc { item: Some(init) },
        }
    }

    fn for_each<F, C>(self, func: F) -> for_each::ForEach<Self, F, C>
    where
        Self: Sized,
        Self: Effective<Produces = Multiple>,
        F: FnMut(Self::Item) -> C,
        C: Effective<Item = ()>,
    {
        for_each::ForEach {
            inner: self,
            func,
            state: for_each::State::Acc,
        }
    }
}

impl<E: Effective> EffectiveExt for E {}

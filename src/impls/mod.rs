use std::convert::Infallible;

use crate::{
    private::{IsAsyncWith, ProducesMultipleWith},
    wrappers::FromTry,
    Async, Blocking, Effective, Multiple, Shim, Single, Try,
};

use self::block::Executor;

pub mod block;
pub mod collect;
pub mod flatten;
pub mod map;
pub mod unwrap;

type FromTryFn<T> = fn(T) -> FromTry<T>;

pub trait EffectiveExt: Effective {
    fn flat_map<R, F>(self, f: F) -> flatten::Flatten<map::Map<Self, F>>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> R,
        R: Effective,
        Self::Produces: ProducesMultipleWith<<R as Effective>::Produces>,
        Self::Async: IsAsyncWith<<R as Effective>::Async>,
        Self::Failure: Into<<R as Effective>::Failure>,
    {
        self.map(f).flatten()
    }

    fn flatten_try(self) -> flatten::FlattenError<map::Map<Self, FromTryFn<Self::Item>>>
    where
        Self: Sized,
        Self: Effective<Failure = Infallible>,
        Self::Item: Try,
        Self::Produces: ProducesMultipleWith<<FromTry<Self::Item> as Effective>::Produces>,
        Self::Async: IsAsyncWith<<FromTry<Self::Item> as Effective>::Async>,
    {
        self.map(crate::wrappers::from_try as _).flatten_error()
    }

    /// Map the items in the effective
    ///
    /// # Example
    ///
    /// ## Try:
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, TryGet, wrappers};
    /// let e = wrappers::from_try(Some(42));
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

    /// Flatten the items in the effective
    ///
    /// # Example
    ///
    /// ## Iterators:
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, wrappers};
    /// let e = wrappers::iterator([1, 2, 3, 4])
    ///     // map to return a sub iterator
    ///     .map(|x| wrappers::iterator(std::iter::repeat(x).take(x)));
    ///
    /// let v: Vec<usize> = e.flatten().collect().get();
    /// assert_eq!(v, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4]);
    /// ```
    ///
    /// ## Futures:
    ///
    /// ```
    /// # async fn foo() {
    /// use effective::{impls::EffectiveExt, wrappers};
    /// let e = wrappers::future(async { 1 })
    ///     // map to return a sub future
    ///     .map(|x| wrappers::future(async move { x + 1 }));
    ///
    /// let v: i32 = e.flatten().shim().await;
    /// assert_eq!(v, 2);
    /// # }
    /// ```
    ///
    /// ## Combined:
    ///
    /// ```
    /// # async fn foo() {
    /// use effective::{impls::EffectiveExt, wrappers};
    /// let e = wrappers::iterator([1, 2, 3, 4])
    ///     // map to return a sub future
    ///     .map(|x| wrappers::future(async move { x + 1 }));
    ///
    /// let v: Vec<usize> = e.flatten().collect().shim().await;
    /// assert_eq!(v, [2, 3, 4, 5]);
    /// # }
    /// ```
    fn flatten(self) -> flatten::Flatten<Self>
    where
        Self: Sized,
        Self::Item: Effective,
        Self::Produces: ProducesMultipleWith<<Self::Item as Effective>::Produces>,
        Self::Async: IsAsyncWith<<Self::Item as Effective>::Async>,
        Self::Failure: Into<<Self::Item as Effective>::Failure>,
    {
        flatten::Flatten {
            inner: self,
            flatten: None,
        }
    }

    fn flatten_error(self) -> flatten::FlattenError<Self>
    where
        Self: Sized,
        Self: Effective<Failure = Infallible>,
        Self::Item: Effective,
        Self::Produces: ProducesMultipleWith<<Self::Item as Effective>::Produces>,
        Self::Async: IsAsyncWith<<Self::Item as Effective>::Async>,
    {
        flatten::FlattenError {
            inner: self,
            flatten: None,
        }
    }

    fn flatten_no_error(self) -> flatten::FlattenNoError<Self>
    where
        Self: Sized,
        Self::Item: Effective<Failure = Infallible>,
        Self::Produces: ProducesMultipleWith<<Self::Item as Effective>::Produces>,
        Self::Async: IsAsyncWith<<Self::Item as Effective>::Async>,
    {
        flatten::FlattenNoError {
            inner: self,
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
    /// use effective::impls::block::FuturesExecutor;
    /// let exec = FuturesExecutor::default();
    ///
    /// let e = wrappers::iterator([1, 2, 3, 4])
    ///     .map(|x| async move { x + 1 })
    ///     .flat_map(wrappers::future);
    ///
    /// let v: Vec<i32> = e.block(exec).collect().get();
    /// assert_eq!(v, [2, 3, 4, 5]);
    /// ```
    fn block<R>(self, executor: R) -> block::Block<Self, R>
    where
        Self: Sized,
        Self: Effective<Async = Async>,
        R: Executor,
    {
        block::Block {
            inner: self,
            executor,
        }
    }

    /// Block on the async effective
    ///
    /// Can be thought of as subtracting the 'fallable' effect.
    ///
    /// # Example
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, wrappers};
    ///
    /// let e = wrappers::iterator([1_i32, 2, 3, 4])
    ///     .map(|x| x.checked_add(1))
    ///     .flatten_try();
    ///
    /// let v: Vec<i32> = e.unwrap().collect().get();
    /// assert_eq!(v, [2, 3, 4, 5]);
    /// ```
    fn unwrap(self) -> unwrap::Unwrap<Self>
    where
        Self: Sized,
        Self::Failure: std::fmt::Debug,
    {
        unwrap::Unwrap { inner: self }
    }

    fn get(self) -> Self::Item
    where
        Self: Sized,
        Self: Effective<Produces = Single, Failure = Infallible, Async = Blocking>,
    {
        crate::Get::get(self)
    }

    fn shim(self) -> Shim<Self>
    where
        Self: Sized,
    {
        Shim { inner: self }
    }
}

impl<E: Effective> EffectiveExt for E {}

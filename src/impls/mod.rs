use crate::{Effective, Shim, private::Combine};

use self::block::Executor;

pub mod collect;
pub mod flatten;
pub mod map;
pub mod block;
pub mod unwrap;

pub trait EffectiveExt: Effective {
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
    /// let v: i32 = e.map(|x| x + 1).into_shim().await;
    /// # }
    /// ```
    ///
    /// ## Iterators:
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, Get, wrappers};
    /// let e = wrappers::iterator([1, 2, 3, 4].into_iter());
    ///
    /// let v: Vec<i32> = e.map(|x| x + 1).collect().get();
    /// assert_eq!(v, [2, 3, 4, 5]);
    /// ```
    fn map<R, F>(self, f: F) -> map::Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> R,
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
    /// use effective::{impls::EffectiveExt, Get, wrappers};
    /// let e = wrappers::iterator([1, 2, 3, 4].into_iter())
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
    /// let v: i32 = e.flatten().into_shim().await;
    /// assert_eq!(v, 2);
    /// # }
    /// ```
    /// 
    /// ## Combined:
    ///
    /// ```
    /// # async fn foo() {
    /// use effective::{impls::EffectiveExt, wrappers};
    /// let e = wrappers::iterator([1, 2, 3, 4].into_iter())
    ///     // map to return a sub future
    ///     .map(|x| wrappers::future(async move { x + 1 }));
    ///
    /// let v: Vec<usize> = e.flatten().collect().into_shim().await;
    /// assert_eq!(v, [2, 3, 4, 5]);
    /// # }
    /// ```
    fn flatten(self) -> flatten::Flatten<Self>
    where
        Self: Sized,
        Self::Output: Effective,
        Self::Yields: Combine<<Self::Output as Effective>::Yields>,
        Self::Awaits: Combine<<Self::Output as Effective>::Awaits>,
        Self::Residual: Into<<Self::Output as Effective>::Residual>,
    {
        flatten::Flatten {
            inner: self,
            flatten: None,
        }
    }

    fn flatten_error(self) -> flatten::FlattenError<Self>
    where
        Self: Sized,
        Self: Effective<Residual = !>,
        Self::Output: Effective,
        Self::Yields: Combine<<Self::Output as Effective>::Yields>,
        Self::Awaits: Combine<<Self::Output as Effective>::Awaits>,
    {
        flatten::FlattenError {
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
    /// use effective::{impls::EffectiveExt, Get, wrappers};
    /// let e = wrappers::iterator([1, 2, 3, 4].into_iter());
    ///
    /// let v: Vec<i32> = e.collect().get();
    /// ```
    fn collect<C>(self) -> collect::Collect<Self, C>
    where
        Self: Sized,
        Self: Effective<Yields = ()>,
        C: Default + Extend<Self::Output>,
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
    /// use effective::{impls::EffectiveExt, Get, wrappers};
    /// 
    /// use effective::impls::block::FuturesExecutor;
    /// let exec = FuturesExecutor::default();
    /// 
    /// let e = wrappers::iterator([1, 2, 3, 4].into_iter())
    ///     .map(|x| wrappers::future(async move { x + 1 }))
    ///     .flatten();
    ///
    /// let v: Vec<i32> = e.block(exec).collect().get();
    /// assert_eq!(v, [2, 3, 4, 5]);
    /// ```
    fn block<R>(self, executor: R) -> block::Block<Self, R>
    where
        Self: Sized,
        Self: Effective<Awaits = ()>,
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
    /// use effective::{impls::EffectiveExt, Get, wrappers};
    /// 
    /// let e = wrappers::iterator([1_i32, 2, 3, 4].into_iter())
    ///     .map(|x| wrappers::from_try(x.checked_add(1)))
    ///     .flatten_error();
    ///
    /// let v: Vec<i32> = e.unwrap().collect().get();
    /// assert_eq!(v, [2, 3, 4, 5]);
    /// ```
    fn unwrap(self) -> unwrap::Unwrap<Self>
    where
        Self: Sized,
        Self::Residual: std::fmt::Debug,
    {
        unwrap::Unwrap {
            inner: self,
        }
    }

    fn into_shim(self) -> Shim<Self>
    where
        Self: Sized,
    {
        Shim { inner: self }
    }
}

impl<E: Effective> EffectiveExt for E {}

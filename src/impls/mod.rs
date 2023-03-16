use std::ops::{FromResidual, Try};

use crate::{private::MinExists, Effective, Shim};

pub mod collect;
pub mod flatten;
pub mod map;

pub trait EffectiveExt: Effective {
    /// Map the items in the effective
    ///
    /// # Example
    ///
    /// ## Try:
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, TryGet, Okay, wrappers};
    /// let e = wrappers::from_try(Some(42));
    ///
    /// let v: Option<i32> = e.map::<Option<_>, _>(|x| x + 1).try_get();
    /// assert_eq!(v, Some(43));
    /// ```
    ///
    /// ## Futures:
    ///
    /// ```
    /// # async fn foo() {
    /// use effective::{impls::EffectiveExt, Okay, wrappers};
    /// let e = wrappers::future(async { 0 });
    ///
    /// let v: i32 = e.map::<Okay<_>, _>(|x| x + 1).into_shim().await;
    /// # }
    /// ```
    ///
    /// ## Iterators:
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, Get, Okay, wrappers};
    /// let e = wrappers::iterator([1, 2, 3, 4].into_iter());
    ///
    /// let v: Vec<i32> = e.map::<Okay<_>, _>(|x| x + 1).collect::<Okay<_>>().get();
    /// assert_eq!(v, [2, 3, 4, 5]);
    /// ```
    fn map<R, F>(self, f: F) -> map::Map<R, Self, F>
    where
        Self: Sized,
        R: Try + FromResidual<<Self::Item as Try>::Residual>,
        F: FnMut(<Self::Item as Try>::Output) -> R::Output,
    {
        map::Map {
            inner: self,
            map: f,
            _marker: std::marker::PhantomData,
        }
    }

    /// Flatten the items in the effective
    ///
    /// # Example
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, Get, Okay, wrappers};
    /// let e = wrappers::iterator([1, 2, 3, 4].into_iter())
    ///     // map returns a sub-effective that yields multiple items
    ///     .map::<Okay<_>, _>(|x| wrappers::iterator(std::iter::repeat(x).take(x)));
    ///
    /// let v: Vec<usize> = e.flatten_items().collect::<Okay<_>>().get();
    /// assert_eq!(v, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4]);
    /// ```
    fn flatten_items(self) -> flatten::FlattenItems<Self>
    where
        Self: Sized,
        <Self::Item as Try>::Output: Effective<Yields = ()>,
        <<Self::Item as Try>::Output as Effective>::Item:
            Try + FromResidual<<Self::Item as Try>::Residual>,
        <<Self::Item as Try>::Output as Effective>::Awaits: MinExists<Self::Awaits>,
    {
        flatten::FlattenItems {
            inner: self,
            flatten: None,
        }
    }

    fn flatten(self) -> flatten::Flatten<Self>
    where
        Self: Sized,
        <Self::Item as Try>::Output: Try + FromResidual<<Self::Item as Try>::Residual>,
    {
        flatten::Flatten { inner: self }
    }

    fn flatten_okay(self) -> flatten::FlattenOkay<Self>
    where
        Self: Sized,
        Self::Item: Try<Residual = !>,
        <Self::Item as Try>::Output: Try,
    {
        flatten::FlattenOkay { inner: self }
    }

    /// Collect the items from this iterator into a collection
    ///
    /// # Example
    ///
    /// ```
    /// use effective::{impls::EffectiveExt, Get, Okay, wrappers};
    /// let e = wrappers::iterator([1, 2, 3, 4].into_iter());
    ///
    /// let v: Vec<i32> = e.collect::<Okay<_>>().get();
    /// ```
    fn collect<R>(self) -> collect::Collect<Self, R>
    where
        Self: Sized,
        Self: Effective<Yields = ()>,
        R: Try + FromResidual<<Self::Item as Try>::Residual>,
        R::Output: Default + Extend<<Self::Item as Try>::Output>,
    {
        collect::Collect {
            inner: self,
            into: Default::default(),
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

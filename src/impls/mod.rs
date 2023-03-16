use crate::{Effective, Shim};

pub mod collect;
// pub mod flatten;
pub mod map;

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

    // /// Flatten the items in the effective
    // ///
    // /// # Example
    // ///
    // /// ```
    // /// use effective::{impls::EffectiveExt, Get, Okay, wrappers};
    // /// let e = wrappers::iterator([1, 2, 3, 4].into_iter())
    // ///     // map returns a sub-effective that yields multiple items
    // ///     .map(|x| wrappers::iterator(std::iter::repeat(x).take(x)));
    // ///
    // /// let v: Vec<usize> = e.flatten_items().collect().get();
    // /// assert_eq!(v, [1, 2, 2, 3, 3, 3, 4, 4, 4, 4]);
    // /// ```
    // fn flatten(self) -> flatten::Flatten<Self>
    // where
    //     Self: Sized,
    //     Self::Output: Effective<Yields = (), Residual = Self::Residual>,
    //     <Self::Output as Effective>::Awaits: MinExists<Self::Awaits>,
    // {
    //     flatten::Flatten {
    //         inner: self,
    //         flatten: None,
    //     }
    // }

    // fn flatten(self) -> flatten::Flatten<Self>
    // where
    //     Self: Sized,
    //     Self::Output: Try + FromResidual<<Self::Item as Try>::Residual>,
    // {
    //     flatten::Flatten { inner: self }
    // }

    // fn flatten_okay(self) -> flatten::FlattenOkay<Self>
    // where
    //     Self: Sized,
    //     Self::Item: Try<Residual = !>,
    //     Self::Output: Try,
    // {
    //     flatten::FlattenOkay { inner: self }
    // }

    /// Collect the items from this iterator into a collection
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

    fn into_shim(self) -> Shim<Self>
    where
        Self: Sized,
    {
        Shim { inner: self }
    }
}

impl<E: Effective> EffectiveExt for E {}

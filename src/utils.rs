//! Some helpers for implementing generic adaptors

use std::convert::Infallible;

use crate::{
    Async, Asynchrony, Blocking, Effective, Failure, Fallible, Iterable, Multiple, Single,
};

use crate::SealedMarker;

/// Represents the asynchrony of a flattened effective.
///
/// # Examples:
///
/// * `blocking(async { 1 })` repsents a [`Blocking`] with [`Async`] inside, so it overall represents [`Async`].
/// * `blocking(blocking(1))` represents a [`Blocking`] with [`Blocking`] inside, so it overall represents [`Blocking`]
pub trait AsyncWith<Rhs>: Sized {
    type IsAsync: Asynchrony;
    fn into_async(self) -> Self::IsAsync {
        <Self::IsAsync>::new()
    }
    fn from_async(_: Rhs) -> Self::IsAsync {
        <Self::IsAsync>::new()
    }
}

impl AsyncWith<Async> for Async {
    type IsAsync = Async;
}

impl AsyncWith<Async> for Blocking {
    type IsAsync = Async;
}

impl AsyncWith<Blocking> for Async {
    type IsAsync = Async;
}

impl AsyncWith<Blocking> for Blocking {
    type IsAsync = Blocking;
}

/// Represents the minimum number of output items from a flattened effective.
///
/// # Examples:
///
/// * `once([1, 2, 3, 4])` repsents a [`Single`] with [`Multiple`] inside, so it overall represents [`Multiple`].
/// * `once(once(1))` represents a [`Single`] with [`Single`] inside, so it overall represents [`Single`]
pub trait IterableWith<Rhs>: Sized {
    type IsIterable: Iterable;
}

impl IterableWith<Multiple> for Multiple {
    type IsIterable = Multiple;
}

impl IterableWith<Multiple> for Single {
    type IsIterable = Multiple;
}

impl IterableWith<Single> for Multiple {
    type IsIterable = Multiple;
}

impl IterableWith<Single> for Single {
    type IsIterable = Single;
}

/// Represents the fallibility a flattened effective.
pub trait FallibleWith<Failure>: Sized {
    type Failure: Fallible;
    fn into_fail(self) -> Self::Failure;
    fn from_fail(_: Failure) -> Self::Failure;
}

impl<F, U> FallibleWith<Failure<U>> for Failure<F>
where
    U: From<F>,
{
    type Failure = Failure<U>;
    fn into_fail(self) -> Self::Failure {
        Failure(self.0.into())
    }
    fn from_fail(x: Failure<U>) -> Self::Failure {
        x
    }
}

impl<F> FallibleWith<Failure<F>> for Infallible {
    type Failure = Failure<F>;
    fn into_fail(self) -> Self::Failure {
        unreachable!()
    }
    fn from_fail(x: Failure<F>) -> Self::Failure {
        x
    }
}

impl<F> FallibleWith<Infallible> for Failure<F> {
    type Failure = Failure<F>;
    fn into_fail(self) -> Self::Failure {
        self
    }
    fn from_fail(_: Infallible) -> Self::Failure {
        unreachable!()
    }
}

impl FallibleWith<Infallible> for Infallible {
    type Failure = Infallible;
    fn into_fail(self) -> Self::Failure {
        self
    }
    fn from_fail(x: Infallible) -> Self::Failure {
        x
    }
}

pub type FalliblePair<E1, E2> =
    <<E1 as Effective>::Failure as FallibleWith<<E2 as Effective>::Failure>>::Failure;
pub type AsyncPair<E1, E2> =
    <<E1 as Effective>::Async as AsyncWith<<E2 as Effective>::Async>>::IsAsync;
pub type IterablePair<E1, E2> =
    <<E1 as Effective>::Produces as IterableWith<<E2 as Effective>::Produces>>::IsIterable;

pub(crate) fn from_fail<E1: Effective, E2: Effective>(x: E2::Failure) -> FalliblePair<E1, E2>
where
    E1::Failure: FallibleWith<E2::Failure>,
{
    <E1::Failure as FallibleWith<E2::Failure>>::from_fail(x)
}

pub(crate) fn from_async<E1: Effective, E2: Effective>(x: E2::Async) -> AsyncPair<E1, E2>
where
    E1::Async: AsyncWith<E2::Async>,
{
    <E1::Async as AsyncWith<E2::Async>>::from_async(x)
}

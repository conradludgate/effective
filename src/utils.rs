//! Some helpers for implementing generic adaptors

use std::convert::Infallible;

use crate::{Async, Asynchronous, Blocking, Fails, Failure, Multiple, Produces, Single};

use crate::SealedMarker;

/// Represents the asynchrony of a flattened effective.
///
/// # Examples:
///
/// * `blocking(async { 1 })` repsents a [`Blocking`] with [`Async`] inside, so it overall represents [`Async`].
/// * `blocking(blocking(1))` represents a [`Blocking`] with [`Blocking`] inside, so it overall represents [`Blocking`]
pub trait IsAsyncWith<Rhs>: Sized {
    type IsAsync: Asynchronous;
    fn into_async(self) -> Self::IsAsync {
        <Self::IsAsync>::new()
    }
    fn from_async(_: Rhs) -> Self::IsAsync {
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

/// Represents the minimum number of output items from a flattened effective.
///
/// # Examples:
///
/// * `once([1, 2, 3, 4])` repsents a [`Single`] with [`Multiple`] inside, so it overall represents [`Multiple`].
/// * `once(once(1))` represents a [`Single`] with [`Single`] inside, so it overall represents [`Single`]
pub trait ProducesMultipleWith<Rhs>: Sized {
    type Produces: Produces;
    fn into_many(self) -> Self::Produces {
        <Self::Produces>::new()
    }
    fn from_many(_: Rhs) -> Self::Produces {
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

/// Represents the fallibility a flattened effective.
pub trait HasFailureWith<Failure>: Sized {
    type Failure: Fails;
    fn into_fail(self) -> Self::Failure;
    fn from_fail(_: Failure) -> Self::Failure;
}

impl<F, U> HasFailureWith<Failure<U>> for Failure<F>
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

impl<F> HasFailureWith<Failure<F>> for Infallible {
    type Failure = Failure<F>;
    fn into_fail(self) -> Self::Failure {
        unreachable!()
    }
    fn from_fail(x: Failure<F>) -> Self::Failure {
        x
    }
}

impl<F> HasFailureWith<Infallible> for Failure<F> {
    type Failure = Failure<F>;
    fn into_fail(self) -> Self::Failure {
        self
    }
    fn from_fail(_: Infallible) -> Self::Failure {
        unreachable!()
    }
}

impl HasFailureWith<Infallible> for Infallible {
    type Failure = Infallible;
    fn into_fail(self) -> Self::Failure {
        self
    }
    fn from_fail(x: Infallible) -> Self::Failure {
        x
    }
}

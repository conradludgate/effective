use std::{convert::Infallible, pin::Pin, task::Context};

use crate::{Blocking, EffectResult, Effective, Multiple};

/// Create an [`Effective`] that has no failures, multiple values and no async
pub fn iterator<I: IntoIterator>(iterator: I) -> FromIterator<I::IntoIter> {
    FromIterator {
        inner: iterator.into_iter(),
    }
}

pin_project_lite::pin_project!(
    pub struct FromIterator<I> {
        pub inner: I,
    }
);

impl<I: Iterator> Effective for FromIterator<I> {
    type Item = I::Item;
    type Failure = Infallible;
    type Produces = Multiple;
    type Async = Blocking;

    fn poll_effect(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Failure, Self::Produces, Self::Async> {
        match self.project().inner.next() {
            Some(x) => EffectResult::Item(x),
            None => EffectResult::Done(Multiple),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    where
        Self: Effective<Produces = Multiple>,
    {
        self.inner.size_hint()
    }
}

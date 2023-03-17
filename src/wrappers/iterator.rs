use std::{pin::Pin, task::Context, convert::Infallible};

use crate::{EffectResult, Effective, Multiple, Blocking};

pub fn iterator<I: IntoIterator>(iterator: I) -> IteratorShim<I::IntoIter> {
    IteratorShim { inner: iterator.into_iter() }
}

pin_project_lite::pin_project!(
    pub struct IteratorShim<I> {
        pub inner: I,
    }
);

impl<I: Iterator> Effective for IteratorShim<I> {
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
}

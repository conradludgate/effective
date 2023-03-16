use std::{pin::Pin, task::Context};

use crate::{EffectResult, Effective, Okay};

pub fn iterator<I>(iterator: I) -> IteratorShim<I> {
    IteratorShim { inner: iterator }
}

pin_project_lite::pin_project!(
    pub struct IteratorShim<I> {
        pub inner: I,
    }
);

impl<I: Iterator> Effective for IteratorShim<I> {
    type Item = Okay<I::Item>;
    type Yields = ();
    type Awaits = !;

    fn poll_effect(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Yields, Self::Awaits> {
        match self.project().inner.next() {
            Some(x) => EffectResult::Item(Okay(x)),
            None => EffectResult::Done(()),
        }
    }
}

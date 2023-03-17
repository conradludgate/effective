use std::{pin::Pin, task::Context, convert::Infallible};

use crate::{EffectResult, Effective};

pin_project_lite::pin_project!(
    /// Produced by the [`unwrap()`](super::EffectiveExt::unwrap) method
    pub struct Unwrap<E> {
        #[pin]
        pub(super) inner: E,
    }
);

impl<E> Effective for Unwrap<E>
where
    E: Effective,
    E::Failure: std::fmt::Debug,
{
    type Item = E::Item;
    type Failure = Infallible;
    type Produces = E::Produces;
    type Async = E::Async;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Failure, Self::Produces, Self::Async> {
        let mut this = self.project();
        match this.inner.as_mut().poll_effect(cx) {
            EffectResult::Item(x) => EffectResult::Item(x),
            EffectResult::Failure(x) => panic!("{x:?}"),
            EffectResult::Done(x) => EffectResult::Done(x),
            EffectResult::Pending(x) => EffectResult::Pending(x),
        }
    }
}

//! Effect adaptors that convert an effect of one type into an effect of another

use std::{pin::Pin, task::Context};

use crate::{EffectResult, Effective};

pin_project_lite::pin_project!(
    /// Produced by the [`map()`](super::EffectiveExt::map) method
    pub struct Map<E, F> {
        #[pin]
        pub(super) inner: E,
        pub(super) map: F,
    }
);

impl<R, E, F> Effective for Map<E, F>
where
    E: Effective,
    F: FnMut(E::Item) -> R,
{
    type Item = R;
    type Failure = E::Failure;
    type Produces = E::Produces;
    type Async = E::Async;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Failure, Self::Produces, Self::Async> {
        let this = self.project();
        match this.inner.poll_effect(cx) {
            EffectResult::Item(x) => EffectResult::Item((this.map)(x)),
            EffectResult::Failure(x) => EffectResult::Failure(x),
            EffectResult::Done(x) => EffectResult::Done(x),
            EffectResult::Pending(x) => EffectResult::Pending(x),
        }
    }
}

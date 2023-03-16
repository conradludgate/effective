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
    F: FnMut(E::Output) -> R,
{
    type Output = R;
    type Residual = E::Residual;
    type Yields = E::Yields;
    type Awaits = E::Awaits;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Output, Self::Residual, Self::Yields, Self::Awaits> {
        let this = self.project();
        match this.inner.poll_effect(cx) {
            EffectResult::Item(x) => EffectResult::Item((this.map)(x)),
            EffectResult::Failure(x) => EffectResult::Failure(x),
            EffectResult::Done(x) => EffectResult::Done(x),
            EffectResult::Pending(x) => EffectResult::Pending(x),
        }
    }
}

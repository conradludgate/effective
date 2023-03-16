use std::{pin::Pin, task::Context};

use crate::{EffectResult, Effective};

pin_project_lite::pin_project!(
    /// Produced by the [`collect()`](super::EffectiveExt::collect) method
    pub struct Collect<E, C> {
        #[pin]
        pub(super) inner: E,
        pub(super) into: C,
    }
);

impl<E, C> Effective for Collect<E, C>
where
    E: Effective<Yields = ()>,
    C: Default + Extend<E::Output>,
{
    type Output = C;
    type Residual = E::Residual;
    type Yields = !;
    type Awaits = E::Awaits;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Output, Self::Residual, Self::Yields, Self::Awaits> {
        let mut this = self.project();
        loop {
            match this.inner.as_mut().poll_effect(cx) {
                EffectResult::Item(x) => this.into.extend(Some(x)),
                EffectResult::Failure(x) => return EffectResult::Failure(x),
                EffectResult::Done(()) => return EffectResult::Item(std::mem::take(this.into)),
                EffectResult::Pending(x) => return EffectResult::Pending(x),
            }
        }
    }
}

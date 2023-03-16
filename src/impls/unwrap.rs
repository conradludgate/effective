use std::{pin::Pin, task::Context};

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
    E::Residual: std::fmt::Debug,
{
    type Output = E::Output;
    type Residual = !;
    type Yields = E::Yields;
    type Awaits = E::Awaits;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Output, Self::Residual, Self::Yields, Self::Awaits> {
        let mut this = self.project();
        match this.inner.as_mut().poll_effect(cx) {
            EffectResult::Item(x) => EffectResult::Item(x),
            EffectResult::Failure(x) => panic!("{x:?}"),
            EffectResult::Done(x) => EffectResult::Done(x),
            EffectResult::Pending(x) => EffectResult::Pending(x),
        }
    }
}

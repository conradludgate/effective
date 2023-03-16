use std::{
    ops::{ControlFlow, FromResidual, Try},
    pin::Pin,
    task::Context,
};

use crate::{EffectResult, Effective};

pin_project_lite::pin_project!(
    /// Produced by the [`collect()`](super::EffectiveExt::collect) method
    pub struct Collect<E, R>
    where
        R: Try,
    {
        #[pin]
        pub(super) inner: E,
        pub(super) into: R::Output,
    }
);

impl<E, R> Effective for Collect<E, R>
where
    E: Effective<Yields = ()>,
    R: Try + FromResidual<<E::Item as Try>::Residual>,
    R::Output: Default + Extend<<E::Item as Try>::Output>,
{
    type Item = R;
    type Yields = !;
    type Awaits = E::Awaits;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Yields, Self::Awaits> {
        let mut this = self.project();
        loop {
            match this.inner.as_mut().poll_effect(cx) {
                EffectResult::Item(x) => match x.branch() {
                    ControlFlow::Continue(x) => this.into.extend(Some(x)),
                    ControlFlow::Break(x) => return EffectResult::Item(R::from_residual(x)),
                },
                EffectResult::Done(()) => {
                    return EffectResult::Item(R::from_output(std::mem::take(this.into)))
                }
                EffectResult::Pending(x) => return EffectResult::Pending(x),
            }
        }
    }
}

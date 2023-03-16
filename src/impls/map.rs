use std::{
    marker::PhantomData,
    ops::{ControlFlow, Try, FromResidual},
    pin::Pin,
    task::Context,
};

use crate::{EffectResult, Effective};

pin_project_lite::pin_project!(
    /// Produced by the [`map()`](super::EffectiveExt::map) method
    pub struct Map<R, E, F> {
        #[pin]
        pub(super) inner: E,
        pub(super) map: F,
        pub(super) _marker: PhantomData<R>,
    }
);

impl<R, E, F> Effective for Map<R, E, F>
where
    E: Effective,
    R: Try + FromResidual<<E::Item as Try>::Residual>,
    F: FnMut(<E::Item as Try>::Output) -> R::Output,
{
    type Item = R;
    type Yields = E::Yields;
    type Awaits = E::Awaits;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Yields, Self::Awaits> {
        let this = self.project();
        match this.inner.poll_effect(cx) {
            EffectResult::Item(x) => match x.branch() {
                ControlFlow::Continue(x) => EffectResult::Item(R::from_output((this.map)(x))),
                ControlFlow::Break(x) => EffectResult::Item(R::from_residual(x)),
            },
            EffectResult::Done(x) => EffectResult::Done(x),
            EffectResult::Pending(x) => EffectResult::Pending(x),
        }
    }
}

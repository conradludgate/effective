use std::{
    ops::{ControlFlow, FromResidual, Try},
    pin::Pin,
    task::Context,
};

use crate::{EffectResult, Effective};

pin_project_lite::pin_project!(
    /// Produced by the [`flatten()`](super::EffectiveExt::flatten) method
    pub struct Flatten<E> {
        #[pin]
        pub(super) inner: E,
    }
);

impl<E> Effective for Flatten<E>
where
    E: Effective,
    <E::Item as Try>::Output: Try + FromResidual<<E::Item as Try>::Residual>,
{
    type Item = <E::Item as Try>::Output;
    type Yields = E::Yields;
    type Awaits = E::Awaits;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Yields, Self::Awaits> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => match x.branch() {
                ControlFlow::Continue(x) => EffectResult::Item(x),
                ControlFlow::Break(x) => EffectResult::Item(<Self::Item as FromResidual<
                    <E::Item as Try>::Residual,
                >>::from_residual(x)),
            },
            EffectResult::Done(x) => EffectResult::Done(x),
            EffectResult::Pending(x) => EffectResult::Pending(x),
        }
    }
}

pin_project_lite::pin_project!(
    /// Produced by the [`flatten()`](super::EffectiveExt::flatten) method
    pub struct FlattenOkay<E> {
        #[pin]
        pub(super) inner: E,
    }
);

impl<E> Effective for FlattenOkay<E>
where
    E: Effective,
    E::Item: Try<Residual = !>,
    <E::Item as Try>::Output: Try,
{
    type Item = <E::Item as Try>::Output;
    type Yields = E::Yields;
    type Awaits = E::Awaits;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Yields, Self::Awaits> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => match x.branch() {
                ControlFlow::Continue(x) => EffectResult::Item(x),
                ControlFlow::Break(_) => unreachable!(),
            },
            EffectResult::Done(x) => EffectResult::Done(x),
            EffectResult::Pending(x) => EffectResult::Pending(x),
        }
    }
}

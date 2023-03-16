use std::{
    ops::{ControlFlow, FromResidual, Try},
    pin::Pin,
    task::Context,
};

use crate::{
    private::{MinExists, Sealed},
    EffectResult, Effective,
};

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
    E::Output: Try + FromResidual<<E::Item as Try>::Residual>,
{
    type Output = E::Output;
    type Yields = E::Yields;
    type Awaits = E::Awaits;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Output, Self::Residual, Self::Yields, Self::Awaits> {
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
    E: Effective<Residual = !>,
    E::Output: Try,
{
    type Output = <E::Output as Try>::Output;
    type Residual = <E::Output as Try>::Residual;
    type Yields = E::Yields;
    type Awaits = E::Awaits;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Output, Self::Residual, Self::Yields, Self::Awaits> {
        match self.project().inner.poll_effect(cx) {
            EffectResult::Item(x) => match x.branch() {
                ControlFlow::Continue(x) => EffectResult::Item(x),
                ControlFlow::Break(x) => EffectResult::Failure(x),
            },
            EffectResult::Failure(_) => unreachable!(),
            EffectResult::Done(x) => EffectResult::Done(x),
            EffectResult::Pending(x) => EffectResult::Pending(x),
        }
    }
}

pin_project_lite::pin_project!(
    /// Produced by the [`flatten()`](super::EffectiveExt::flatten) method
    pub struct FlattenItems<E>
    where
        E: Effective,
    {
        #[pin]
        pub(super) inner: E,
        #[pin]
        pub(super) flatten: Option<E::Output>,
    }
);

impl<E> Effective for FlattenItems<E>
where
    E: Effective,
    E::Output: Effective<Yields = ()>,
    <E::Output as Effective>::Item: Try + FromResidual<<E::Item as Try>::Residual>,
    <E::Output as Effective>::Awaits: MinExists<E::Awaits>,
{
    type Item = <E::Output as Effective>::Item;
    type Yields = E::Yields;
    type Awaits = <<E::Output as Effective>::Awaits as MinExists<E::Awaits>>::Exists;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Output, Self::Residual, Self::Yields, Self::Awaits> {
        let mut this = self.project();
        loop {
            if let Some(flatten) = this.flatten.as_mut().as_pin_mut() {
                match flatten.poll_effect(cx) {
                    EffectResult::Done(_) => this.flatten.set(None),
                    EffectResult::Item(x) => return EffectResult::Item(x),
                    EffectResult::Pending(_) => {
                        return EffectResult::Pending(<Self::Awaits as Sealed>::new())
                    }
                }
            }

            match this.inner.as_mut().poll_effect(cx) {
                EffectResult::Item(x) => match x.branch() {
                    ControlFlow::Continue(x) => this.flatten.set(Some(x)),
                    ControlFlow::Break(x) => {
                        return EffectResult::Item(FromResidual::from_residual(x))
                    }
                },
                EffectResult::Done(x) => return EffectResult::Done(x),
                EffectResult::Pending(_) => {
                    return EffectResult::Pending(<Self::Awaits as Sealed>::new())
                }
            }
        }
    }
}

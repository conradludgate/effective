use std::{pin::Pin, task::Context};

use crate::{private::Combine, EffectResult, Effective, Exists};

pin_project_lite::pin_project!(
    /// Produced by the [`flatten()`](super::EffectiveExt::flatten) method
    pub struct Flatten<E>
    where
        E: Effective,
    {
        #[pin]
        pub(super) inner: E,
        #[pin]
        pub(super) flatten: Option<E::Output>,
    }
);

impl<E> Effective for Flatten<E>
where
    E: Effective,
    E::Output: Effective,
    E::Yields: Combine<<E::Output as Effective>::Yields>,
    E::Awaits: Combine<<E::Output as Effective>::Awaits>,
    E::Residual: Into<<E::Output as Effective>::Residual>,
{
    type Output = <E::Output as Effective>::Output;
    type Yields = <E::Yields as Combine<<E::Output as Effective>::Yields>>::Max;
    type Awaits = <E::Awaits as Combine<<E::Output as Effective>::Awaits>>::Max;
    type Residual = <E::Output as Effective>::Residual;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Output, Self::Residual, Self::Yields, Self::Awaits> {
        let mut this = self.project();
        loop {
            if let Some(flatten) = this.flatten.as_mut().as_pin_mut() {
                match flatten.poll_effect(cx) {
                    EffectResult::Done(_) => this.flatten.set(None),
                    EffectResult::Failure(x) => return EffectResult::Failure(x),
                    EffectResult::Item(x) => {
                        if !<<E::Output as Effective>::Yields as Exists>::EXISTS {
                            this.flatten.set(None);
                        }
                        return EffectResult::Item(x);
                    }
                    EffectResult::Pending(x) => {
                        return EffectResult::Pending(<E::Awaits as Combine<
                            <E::Output as Effective>::Awaits,
                        >>::from_rhs(x))
                    }
                }
            }

            match this.inner.as_mut().poll_effect(cx) {
                EffectResult::Item(x) => {
                    this.flatten.set(Some(x))
                },
                EffectResult::Failure(x) => return EffectResult::Failure(x.into()),
                EffectResult::Done(x) => return EffectResult::Done(x.into_max()),
                EffectResult::Pending(x) => return EffectResult::Pending(x.into_max()),
            }
        }
    }
}

pin_project_lite::pin_project!(
    /// Produced by the [`flatten()`](super::EffectiveExt::flatten) method
    pub struct FlattenError<E>
    where
        E: Effective,
    {
        #[pin]
        pub(super) inner: E,
        #[pin]
        pub(super) flatten: Option<E::Output>,
    }
);

impl<E> Effective for FlattenError<E>
where
    E: Effective<Residual = !>,
    E::Output: Effective,
    E::Yields: Combine<<E::Output as Effective>::Yields>,
    E::Awaits: Combine<<E::Output as Effective>::Awaits>,
{
    type Output = <E::Output as Effective>::Output;
    type Yields = <E::Yields as Combine<<E::Output as Effective>::Yields>>::Max;
    type Awaits = <E::Awaits as Combine<<E::Output as Effective>::Awaits>>::Max;
    type Residual = <E::Output as Effective>::Residual;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Output, Self::Residual, Self::Yields, Self::Awaits> {
        let mut this = self.project();
        loop {
            if let Some(flatten) = this.flatten.as_mut().as_pin_mut() {
                match flatten.poll_effect(cx) {
                    EffectResult::Done(_) => this.flatten.set(None),
                    EffectResult::Failure(x) => return EffectResult::Failure(x),
                    EffectResult::Item(x) => {
                        if !<<E::Output as Effective>::Yields as Exists>::EXISTS {
                            this.flatten.set(None);
                        }
                        return EffectResult::Item(x)
                    },
                    EffectResult::Pending(x) => {
                        return EffectResult::Pending(<E::Awaits as Combine<
                            <E::Output as Effective>::Awaits,
                        >>::from_rhs(x))
                    }
                }
            }

            match this.inner.as_mut().poll_effect(cx) {
                EffectResult::Item(x) => this.flatten.set(Some(x)),
                EffectResult::Failure(_) => unreachable!(),
                EffectResult::Done(x) => return EffectResult::Done(x.into_max()),
                EffectResult::Pending(x) => return EffectResult::Pending(x.into_max()),
            }
        }
    }
}

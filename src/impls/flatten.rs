use std::{pin::Pin, task::Context};

use crate::{private::Sealed, EffectResult, Effective};

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
    E::Output: Effective<Awaits = E::Awaits, Yields = (), Residual = E::Residual>,
{
    type Output = <E::Output as Effective>::Output;
    type Yields = ();
    type Awaits = E::Awaits;
    type Residual = E::Residual;

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
                    EffectResult::Item(x) => return EffectResult::Item(x),
                    EffectResult::Pending(x) => return EffectResult::Pending(x),
                }
            }

            match this.inner.as_mut().poll_effect(cx) {
                EffectResult::Item(x) => this.flatten.set(Some(x)),
                EffectResult::Failure(x) => return EffectResult::Failure(x),
                EffectResult::Done(_) => return EffectResult::Done(()),
                EffectResult::Pending(_) => {
                    return EffectResult::Pending(<Self::Awaits as Sealed>::new())
                }
            }
        }
    }
}

//! Effect adaptors to subtract the 'iterable' effect

use std::{pin::Pin, task::Context};

use crate::{EffectResult, Effective, Multiple, Single, Asynchronous, Produces};

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
    E: Effective<Produces = Multiple>,
    C: Default + Extend<E::Item>,
{
    type Item = C;
    type Failure = E::Failure;
    type Produces = Single;
    type Async = E::Async;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Failure, Self::Produces, Self::Async> {
        if !<Self::Async as Asynchronous>::IS_ASYNC && !<Self::Produces as Produces>::MULTIPLE {
            
        }

        let mut this = self.project();
        loop {
            match this.inner.as_mut().poll_effect(cx) {
                EffectResult::Item(x) => this.into.extend(Some(x)),
                EffectResult::Failure(x) => return EffectResult::Failure(x),
                EffectResult::Done(Multiple) => {
                    return EffectResult::Item(std::mem::take(this.into))
                }
                EffectResult::Pending(x) => return EffectResult::Pending(x),
            }
        }
    }
}

//! Effect adaptors to subtract the 'fallible' effect

use std::{convert::Infallible, pin::Pin, task::Context};

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
    E::Failure: std::fmt::Debug,
{
    type Item = E::Item;
    type Failure = Infallible;
    type Produces = E::Produces;
    type Async = E::Async;

    fn poll_effect(self: Pin<&mut Self>, cx: &mut Context<'_>) -> crate::EffectiveResult<Self> {
        let mut this = self.project();
        match this.inner.as_mut().poll_effect(cx) {
            EffectResult::Item(x) => EffectResult::Item(x),
            EffectResult::Failure(x) => {
                panic!("called `EffectiveExt::unwrap()` on an `Failure`: value {x:?}")
            }
            EffectResult::Done(x) => EffectResult::Done(x),
            EffectResult::Pending(x) => EffectResult::Pending(x),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

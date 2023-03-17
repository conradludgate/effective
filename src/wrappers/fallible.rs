use std::{pin::Pin, task::Context};

use crate::{Blocking, EffectResult, Effective, Fallible, Single};

/// Create an [`Effective`] that has a failure, a single value and no async
pub fn fallible<T>(t: T) -> FromFallible<T> {
    FromFallible { inner: Some(t) }
}

pin_project_lite::pin_project!(
    pub struct FromFallible<T> {
        pub inner: Option<T>,
    }
);

impl<T: Fallible> Effective for FromFallible<T> {
    type Item = T::Continue;
    type Failure = T::Break;
    type Produces = Single;
    type Async = Blocking;

    fn poll_effect(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> EffectResult<T::Continue, T::Break, Single, Blocking> {
        let this = self.project();
        let x = this.inner.take().expect("polled after completion");
        match x.branch() {
            std::ops::ControlFlow::Continue(x) => EffectResult::Item(x),
            std::ops::ControlFlow::Break(x) => EffectResult::Failure(x),
        }
    }
}

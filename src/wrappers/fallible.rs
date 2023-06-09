use std::{pin::Pin, task::Context};

use crate::{Blocking, EffectResult, Effective, EffectiveResult, Failure, SimpleTry, Single};

/// Create an [`Effective`] that has a failure, a single value and no async
pub fn fallible<T>(t: T) -> FromFallible<T> {
    FromFallible { inner: Some(t) }
}

pin_project_lite::pin_project!(
    pub struct FromFallible<T> {
        pub inner: Option<T>,
    }
);

impl<T: SimpleTry> Effective for FromFallible<T> {
    type Item = T::Continue;
    type Failure = Failure<T::Break>;
    type Produces = Single;
    type Async = Blocking;

    fn poll_effect(self: Pin<&mut Self>, _: &mut Context<'_>) -> EffectiveResult<Self> {
        let this = self.project();
        let x = this.inner.take().expect("polled after completion");
        match x.branch() {
            std::ops::ControlFlow::Continue(x) => EffectResult::Item(x),
            std::ops::ControlFlow::Break(x) => EffectResult::Failure(Failure(x)),
        }
    }
}

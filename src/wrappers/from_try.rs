use std::{pin::Pin, task::Context};

use crate::{Blocking, EffectResult, Effective, Single, Try};

pub fn from_try<T>(t: T) -> FromTry<T> {
    FromTry { inner: Some(t) }
}

pin_project_lite::pin_project!(
    pub struct FromTry<T> {
        pub inner: Option<T>,
    }
);

impl<T: Try> Effective for FromTry<T> {
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

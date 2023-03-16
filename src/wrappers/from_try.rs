use std::{ops::Try, pin::Pin, task::Context};

use crate::{EffectResult, Effective};

pub fn from_try<T>(t: T) -> FromTry<T> {
    FromTry { inner: Some(t) }
}

pin_project_lite::pin_project!(
    pub struct FromTry<T> {
        pub inner: Option<T>,
    }
);

impl<T: Try> Effective for FromTry<T> {
    type Output = T::Output;
    type Residual = T::Residual;
    type Yields = !;
    type Awaits = !;

    fn poll_effect(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> EffectResult<T::Output, T::Residual, Self::Yields, Self::Awaits> {
        let this = self.project();
        let x = this.inner.take().expect("polled after completion");
        match x.branch() {
            std::ops::ControlFlow::Continue(x) => EffectResult::Item(x),
            std::ops::ControlFlow::Break(x) => EffectResult::Failure(x),
        }
    }
}

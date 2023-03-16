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
    type Item = T;
    type Yields = !;
    type Awaits = !;

    fn poll_effect(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Yields, Self::Awaits> {
        EffectResult::Item(
            self.project()
                .inner
                .take()
                .expect("polled after completion"),
        )
    }
}

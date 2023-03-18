use std::{convert::Infallible, pin::Pin, task::Context};

use crate::{Blocking, EffectResult, Effective, Single};

/// Create an `Effective` that returns a single value, no failures and no async
pub fn once<T>(t: T) -> Once<T> {
    Once { inner: Some(t) }
}

pin_project_lite::pin_project!(
    pub struct Once<T> {
        pub inner: Option<T>,
    }
);

impl<T> Effective for Once<T> {
    type Item = T;
    type Failure = Infallible;
    type Produces = Single;
    type Async = Blocking;

    fn poll_effect(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> EffectResult<T, Infallible, Single, Blocking> {
        let x = self
            .project()
            .inner
            .take()
            .expect("polled after completion");
        EffectResult::Item(x)
    }
}

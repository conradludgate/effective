use std::{pin::Pin, task::Context, convert::Infallible};

use crate::{EffectResult, Effective, Multiple, Blocking, Single};

pub fn from_fn<F>(f: F) -> FromFn<F> {
    FromFn { inner: f }
}

pub fn from_fn_once<F>(f: F) -> FromFnOnce<F> {
    FromFnOnce { inner: Some(f) }
}

pin_project_lite::pin_project!(
    pub struct FromFn<F> {
        pub inner: F,
    }
);

impl<F, R> Effective for FromFn<F>
where
    F: FnMut() -> R,
{
    type Item = R;
    type Failure = Infallible;
    type Produces = Multiple;
    type Async = Blocking;

    fn poll_effect(self: Pin<&mut Self>, _: &mut Context<'_>) -> EffectResult<R, Infallible, Multiple, Blocking> {
        EffectResult::Item((self.project().inner)())
    }
}

pin_project_lite::pin_project!(
    pub struct FromFnOnce<F> {
        pub inner: Option<F>,
    }
);

impl<F, R> Effective for FromFnOnce<F>
where
    F: FnOnce() -> R,
{
    type Item = R;
    type Failure = Infallible;
    type Produces = Single;
    type Async = Blocking;

    fn poll_effect(self: Pin<&mut Self>, _: &mut Context<'_>) -> EffectResult<R, Infallible, Single, Blocking> {
        let x = self
            .project()
            .inner
            .take()
            .expect("polled after completion");
        EffectResult::Item(x())
    }
}

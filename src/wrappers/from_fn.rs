use std::{convert::Infallible, pin::Pin, task::Context};

use crate::{Asynchronous, Blocking, EffectResult, Effective, Fails, Single};

/// Create a raw `Effective` from a function
pub fn from_fn<F>(f: F) -> FromFn<F> {
    FromFn { inner: f }
}

/// Create an `Effective` that returns a single value, no failures and no async
pub fn from_fn_once<F>(f: F) -> FromFnOnce<F> {
    FromFnOnce { inner: Some(f) }
}

pin_project_lite::pin_project!(
    pub struct FromFn<F> {
        pub inner: F,
    }
);

impl<F, Item, Failure, Produces, Async> Effective for FromFn<F>
where
    F: FnMut(&mut Context<'_>) -> EffectResult<Item, Failure, Produces, Async>,
    Failure: Fails,
    Produces: crate::Produces,
    Async: Asynchronous,
{
    type Item = Item;
    type Failure = Failure;
    type Produces = Produces;
    type Async = Async;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Item, Failure, Produces, Async> {
        (self.project().inner)(cx)
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

    fn poll_effect(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> EffectResult<R, Infallible, Single, Blocking> {
        let x = self
            .project()
            .inner
            .take()
            .expect("polled after completion");
        EffectResult::Item(x())
    }
}

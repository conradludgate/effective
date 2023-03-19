//! Effect adaptors to subtract the 'iterable' effect

use std::{pin::Pin, task::Context};

use crate::{
    utils::{from_async, from_fail, AsyncPair, AsyncWith, FalliblePair, FallibleWith},
    EffectResult, Effective, Multiple, Single,
};

pin_project_lite::pin_project!(
    #[project = StateProj]
    pub(super) enum State<B, C> {
        Acc {
            item: Option<B>,
        },
        Eff {
            #[pin]
            eff: C,
        },
    }
);

pin_project_lite::pin_project!(
    /// Produced by the [`fold()`](super::EffectiveExt::fold) method
    pub struct Fold<E, F, B, C> {
        #[pin]
        pub(super) inner: E,
        pub(super) func: F,
        #[pin]
        pub(super) state: State<B, C>,
    }
);

impl<E, F, B, C> Effective for Fold<E, F, B, C>
where
    E: Effective<Produces = Multiple>,
    F: FnMut(B, E::Item) -> C,
    C: Effective<Item = B, Produces = Single>,
    E::Async: AsyncWith<C::Async>,
    E::Failure: FallibleWith<C::Failure>,
{
    type Item = B;
    type Failure = FalliblePair<E, C>;
    type Produces = Single;
    type Async = AsyncPair<E, C>;

    fn poll_effect(self: Pin<&mut Self>, cx: &mut Context<'_>) -> crate::EffectiveResult<Self> {
        let mut this = self.project();
        loop {
            match this.state.as_mut().project() {
                StateProj::Acc { item } => match this.inner.as_mut().poll_effect(cx) {
                    EffectResult::Item(x) => {
                        let eff = (this.func)(item.take().unwrap(), x);
                        this.state.set(State::Eff { eff });
                    }
                    EffectResult::Failure(x) => return EffectResult::Failure(x.into_fail()),
                    EffectResult::Done(Multiple) => {
                        return EffectResult::Item(item.take().unwrap())
                    }
                    EffectResult::Pending(x) => return EffectResult::Pending(x.into_async()),
                },
                StateProj::Eff { eff } => match eff.poll_effect(cx) {
                    EffectResult::Item(item) => this.state.set(State::Acc { item: Some(item) }),
                    EffectResult::Done(_) => unreachable!(),
                    EffectResult::Failure(x) => return EffectResult::Failure(from_fail::<E, C>(x)),
                    EffectResult::Pending(x) => {
                        return EffectResult::Pending(from_async::<E, C>(x))
                    }
                },
            }
        }
    }
}

impl<E, F, B, C> std::future::Future for Fold<E, F, B, C>
where
    E: Effective<Produces = Multiple>,
    F: FnMut(B, E::Item) -> C,
    C: Effective<Item = B, Produces = Single>,
    E::Async: AsyncWith<C::Async>,
    E::Failure: FallibleWith<C::Failure>,
{
    type Output = B;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        match self.poll_effect(cx) {
            EffectResult::Item(value) => std::task::Poll::Ready(value),
            EffectResult::Failure(_) => unreachable!(),
            EffectResult::Done(_) => unreachable!(),
            EffectResult::Pending(_) => std::task::Poll::Pending,
        }
    }
}

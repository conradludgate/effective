//! Effect adaptors to subtract the 'iterable' effect

use std::{pin::Pin, task::Context};

use crate::{
    utils::{from_async, from_fail, AsyncPair, AsyncWith, FalliblePair, FallibleWith},
    EffectResult, Effective, Multiple, Single,
};

pin_project_lite::pin_project!(
    #[project = StateProj]
    pub(super) enum State<C> {
        Acc,
        Eff {
            #[pin]
            eff: C,
        },
    }
);

pin_project_lite::pin_project!(
    /// Produced by the [`for_each()`](super::EffectiveExt::for_each) method
    pub struct ForEach<E, F, C> {
        #[pin]
        pub(super) inner: E,
        pub(super) func: F,
        #[pin]
        pub(super) state: State<C>,
    }
);

impl<E, F, C> Effective for ForEach<E, F, C>
where
    E: Effective<Produces = Multiple>,
    F: FnMut(E::Item) -> C,
    C: Effective<Item = ()>,
    E::Async: AsyncWith<C::Async>,
    E::Failure: FallibleWith<C::Failure>,
{
    type Item = ();
    type Failure = FalliblePair<E, C>;
    type Produces = Single;
    type Async = AsyncPair<E, C>;

    fn poll_effect(self: Pin<&mut Self>, cx: &mut Context<'_>) -> crate::EffectiveResult<Self> {
        let mut this = self.project();
        loop {
            match this.state.as_mut().project() {
                StateProj::Acc => match this.inner.as_mut().poll_effect(cx) {
                    EffectResult::Item(x) => {
                        let eff = (this.func)(x);
                        this.state.set(State::Eff { eff });
                    }
                    EffectResult::Failure(x) => return EffectResult::Failure(x.into_fail()),
                    EffectResult::Done(Multiple) => return EffectResult::Item(()),
                    EffectResult::Pending(x) => return EffectResult::Pending(x.into_async()),
                },
                StateProj::Eff { eff } => match eff.poll_effect(cx) {
                    EffectResult::Item(()) if <C::Produces as crate::Iterable>::MULTIPLE => {}
                    EffectResult::Item(()) | EffectResult::Done(_) => this.state.set(State::Acc),
                    EffectResult::Failure(x) => return EffectResult::Failure(from_fail::<E, C>(x)),
                    EffectResult::Pending(x) => {
                        return EffectResult::Pending(from_async::<E, C>(x))
                    }
                },
            }
        }
    }
}

impl<E, F, C> std::future::Future for ForEach<E, F, C>
where
    E: Effective<Produces = Multiple>,
    F: FnMut(E::Item) -> C,
    C: Effective<Item = ()>,
    E::Async: AsyncWith<C::Async>,
    E::Failure: FallibleWith<C::Failure>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        match self.poll_effect(cx) {
            EffectResult::Item(value) => std::task::Poll::Ready(value),
            EffectResult::Failure(_) => unreachable!(),
            EffectResult::Done(x) => match x {},
            EffectResult::Pending(_) => std::task::Poll::Pending,
        }
    }
}

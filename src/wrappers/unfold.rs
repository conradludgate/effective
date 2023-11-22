use std::{pin::Pin, task::Context};

use crate::{EffectResult, Effective, Multiple, Single};

/// Creates a `Effective` with multiple values from a seed and
/// a closure returning a `Effective` with only a single value.
///
/// This function is the dual for the `EffectiveExt::fold()` adapter: while
/// `EffectiveExt::fold()` reduces a `Effective` to one single value, `unfold()`
/// creates multiple values from a seed value.
///
/// `unfold()` will call the provided closure with the provided seed, then wait
/// for the returned `Effective` to complete with `(a, b)`. It will then yield the
/// value `a`, and use `b` as the next internal state.
///
/// If the effective returns `None` instead of `Some(_)`, then the `unfold()`
/// will stop producing items and return `EffectResult::Done` in later
/// calls to `poll_effect()`.
pub fn unfold<T, F, E, Item>(init: T, f: F) -> Unfold<T, F, E>
where
    F: FnMut(T) -> E,
    E: Effective<Item = Option<(Item, T)>, Produces = Single>,
{
    Unfold {
        func: f,
        state: State::Acc { item: Some(init) },
    }
}

pin_project_lite::pin_project!(
    pub struct Unfold<T, F, E> {
        func: F,
        #[pin]
        state: State<T, E>,
    }
);

pin_project_lite::pin_project!(
    #[project = StateProj]
    pub(super) enum State<T, E> {
        Acc {
            item: Option<T>,
        },
        Eff {
            #[pin]
            eff: E,
        },
    }
);

impl<T, F, E, Item> Effective for Unfold<T, F, E>
where
    F: FnMut(T) -> E,
    E: Effective<Item = Option<(Item, T)>, Produces = Single>,
{
    type Item = Item;
    type Failure = E::Failure;
    type Async = E::Async;
    type Produces = Multiple;

    fn poll_effect(self: Pin<&mut Self>, cx: &mut Context<'_>) -> crate::EffectiveResult<Self> {
        let mut this = self.project();
        loop {
            match this.state.as_mut().project() {
                StateProj::Acc { item } => {
                    let eff = (this.func)(item.take().unwrap());
                    this.state.set(State::Eff { eff });
                }
                StateProj::Eff { eff } => match eff.poll_effect(cx) {
                    EffectResult::Item(Some((item, t))) => {
                        this.state.set(State::Acc { item: Some(t) });
                        return EffectResult::Item(item);
                    }
                    EffectResult::Item(None) => return EffectResult::Done(Multiple),
                    EffectResult::Failure(x) => return EffectResult::Failure(x),
                    EffectResult::Done(x) => match x {},
                    EffectResult::Pending(x) => return EffectResult::Pending(x),
                },
            }
        }
    }
}

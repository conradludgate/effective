//! Effect adaptors that handle effects of effects

use std::{pin::Pin, task::Context};

use crate::{
    utils::{
        from_async, from_fail, AsyncPair, AsyncWith, FalliblePair, FallibleWith, IterablePair,
        IterableWith,
    },
    EffectResult, Effective, Iterable,
};

pin_project_lite::pin_project!(
    /// Produced by the [`flatten()`](super::EffectiveExt::flatten) method
    pub struct Flatten<E>
    where
        E: Effective,
    {
        #[pin]
        pub(super) inner: Option<E>,
        #[pin]
        pub(super) flatten: Option<E::Item>,
    }
);

impl<E> Effective for Flatten<E>
where
    E: Effective,
    E::Item: Effective,
    E::Produces: IterableWith<<E::Item as Effective>::Produces>,
    E::Async: AsyncWith<<E::Item as Effective>::Async>,
    E::Failure: FallibleWith<<E::Item as Effective>::Failure>,
{
    type Item = <E::Item as Effective>::Item;
    type Produces = IterablePair<E, E::Item>;
    type Async = AsyncPair<E, E::Item>;
    type Failure = FalliblePair<E, E::Item>;

    fn poll_effect(self: Pin<&mut Self>, cx: &mut Context<'_>) -> crate::EffectiveResult<Self> {
        let mut this = self.project();
        loop {
            if let Some(flatten) = this.flatten.as_mut().as_pin_mut() {
                match flatten.poll_effect(cx) {
                    EffectResult::Done(_) => this.flatten.set(None),
                    EffectResult::Failure(x) => {
                        return EffectResult::Failure(from_fail::<E, E::Item>(x))
                    }
                    EffectResult::Item(x) => {
                        if !<<E::Item as Effective>::Produces as Iterable>::MULTIPLE {
                            this.flatten.set(None);
                        }
                        return EffectResult::Item(x);
                    }
                    EffectResult::Pending(x) => {
                        return EffectResult::Pending(from_async::<E, E::Item>(x))
                    }
                }
            }

            if let Some(inner) = this.inner.as_mut().as_pin_mut() {
                match inner.poll_effect(cx) {
                    EffectResult::Item(x) => {
                        if !<E::Produces as Iterable>::MULTIPLE {
                            this.inner.set(None);
                        }
                        this.flatten.set(Some(x))
                    }
                    EffectResult::Failure(x) => return EffectResult::Failure(x.into_fail()),
                    EffectResult::Done(_) => this.inner.set(None),
                    EffectResult::Pending(x) => return EffectResult::Pending(x.into_async()),
                }
            } else {
                use crate::SealedMarker;
                return EffectResult::Done(<<E::Produces as IterableWith<
                    <E::Item as Effective>::Produces,
                >>::IsIterable as SealedMarker>::new());
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if <E::Produces as Iterable>::MULTIPLE
            && <<E::Item as Effective>::Produces as Iterable>::MULTIPLE
        {
            (0, None)
        } else if <E::Produces as Iterable>::MULTIPLE {
            if let Some(inner) = self.inner.as_ref() {
                inner.size_hint()
            } else {
                (0, Some(0))
            }
        } else if let Some(flatten) = self.flatten.as_ref() {
            flatten.size_hint()
        } else if self.inner.is_some() {
            (0, None)
        } else {
            (0, Some(0))
        }
    }
}

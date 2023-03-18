//! Effect adaptors that handle effects of effects

use std::{pin::Pin, task::Context};

use crate::{
    utils::{HasFailureWith, IsAsyncWith, ProducesMultipleWith},
    EffectResult, Effective, Produces,
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
    E::Produces: ProducesMultipleWith<<E::Item as Effective>::Produces>,
    E::Async: IsAsyncWith<<E::Item as Effective>::Async>,
    E::Failure: HasFailureWith<<E::Item as Effective>::Failure>,
{
    type Item = <E::Item as Effective>::Item;
    type Produces =
        <E::Produces as ProducesMultipleWith<<E::Item as Effective>::Produces>>::Produces;
    type Async = <E::Async as IsAsyncWith<<E::Item as Effective>::Async>>::IsAsync;
    type Failure = <E::Failure as HasFailureWith<<E::Item as Effective>::Failure>>::Failure;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Failure, Self::Produces, Self::Async> {
        let mut this = self.project();
        loop {
            if let Some(flatten) = this.flatten.as_mut().as_pin_mut() {
                match flatten.poll_effect(cx) {
                    EffectResult::Done(_) => this.flatten.set(None),
                    EffectResult::Failure(x) => {
                        return EffectResult::Failure(<E::Failure as HasFailureWith<
                            <E::Item as Effective>::Failure,
                        >>::from_fail(x))
                    }
                    EffectResult::Item(x) => {
                        if !<<E::Item as Effective>::Produces as Produces>::MULTIPLE {
                            this.flatten.set(None);
                        }
                        return EffectResult::Item(x);
                    }
                    EffectResult::Pending(x) => {
                        return EffectResult::Pending(<E::Async as IsAsyncWith<
                            <E::Item as Effective>::Async,
                        >>::from_async(x))
                    }
                }
            }

            if let Some(inner) = this.inner.as_mut().as_pin_mut() {
                match inner.poll_effect(cx) {
                    EffectResult::Item(x) => {
                        if !<E::Produces as Produces>::MULTIPLE {
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
                return EffectResult::Done(<<E::Produces as ProducesMultipleWith<
                    <E::Item as Effective>::Produces,
                >>::Produces as SealedMarker>::new());
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if <E::Produces as Produces>::MULTIPLE
            && <<E::Item as Effective>::Produces as Produces>::MULTIPLE
        {
            (0, None)
        } else if <E::Produces as Produces>::MULTIPLE {
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

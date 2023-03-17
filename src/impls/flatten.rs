use std::{pin::Pin, task::Context, convert::Infallible};

use crate::{
    private::{IsAsyncWith, ProducesMultipleWith},
    EffectResult, Effective, Produces,
};

pin_project_lite::pin_project!(
    /// Produced by the [`flatten()`](super::EffectiveExt::flatten) method
    pub struct Flatten<E>
    where
        E: Effective,
    {
        #[pin]
        pub(super) inner: E,
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
    E::Failure: Into<<E::Item as Effective>::Failure>,
{
    type Item = <E::Item as Effective>::Item;
    type Produces =
        <E::Produces as ProducesMultipleWith<<E::Item as Effective>::Produces>>::Produces;
    type Async = <E::Async as IsAsyncWith<<E::Item as Effective>::Async>>::IsAsync;
    type Failure = <E::Item as Effective>::Failure;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Failure, Self::Produces, Self::Async> {
        let mut this = self.project();
        loop {
            if let Some(flatten) = this.flatten.as_mut().as_pin_mut() {
                match flatten.poll_effect(cx) {
                    EffectResult::Done(_) => this.flatten.set(None),
                    EffectResult::Failure(x) => return EffectResult::Failure(x),
                    EffectResult::Item(x) => {
                        if !<<E::Item as Effective>::Produces as Produces>::MULTIPLE {
                            this.flatten.set(None);
                        }
                        return EffectResult::Item(x);
                    }
                    EffectResult::Pending(x) => {
                        return EffectResult::Pending(<E::Async as IsAsyncWith<
                            <E::Item as Effective>::Async,
                        >>::from_rhs(x))
                    }
                }
            }

            match this.inner.as_mut().poll_effect(cx) {
                EffectResult::Item(x) => this.flatten.set(Some(x)),
                EffectResult::Failure(x) => return EffectResult::Failure(x.into()),
                EffectResult::Done(x) => return EffectResult::Done(x.into_max()),
                EffectResult::Pending(x) => return EffectResult::Pending(x.into_max()),
            }
        }
    }
}

pin_project_lite::pin_project!(
    /// Produced by the [`flatten_error()`](super::EffectiveExt::flatten_error) method
    pub struct FlattenError<E>
    where
        E: Effective,
    {
        #[pin]
        pub(super) inner: E,
        #[pin]
        pub(super) flatten: Option<E::Item>,
    }
);

impl<E> Effective for FlattenError<E>
where
    E: Effective<Failure = Infallible>,
    E::Item: Effective,
    E::Produces: ProducesMultipleWith<<E::Item as Effective>::Produces>,
    E::Async: IsAsyncWith<<E::Item as Effective>::Async>,
{
    type Item = <E::Item as Effective>::Item;
    type Produces =
        <E::Produces as ProducesMultipleWith<<E::Item as Effective>::Produces>>::Produces;
    type Async = <E::Async as IsAsyncWith<<E::Item as Effective>::Async>>::IsAsync;
    type Failure = <E::Item as Effective>::Failure;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Failure, Self::Produces, Self::Async> {
        let mut this = self.project();
        loop {
            if let Some(flatten) = this.flatten.as_mut().as_pin_mut() {
                match flatten.poll_effect(cx) {
                    EffectResult::Done(_) => this.flatten.set(None),
                    EffectResult::Failure(x) => return EffectResult::Failure(x),
                    EffectResult::Item(x) => {
                        if !<<E::Item as Effective>::Produces as Produces>::MULTIPLE {
                            this.flatten.set(None);
                        }
                        return EffectResult::Item(x);
                    }
                    EffectResult::Pending(x) => {
                        return EffectResult::Pending(<E::Async as IsAsyncWith<
                            <E::Item as Effective>::Async,
                        >>::from_rhs(x))
                    }
                }
            }

            match this.inner.as_mut().poll_effect(cx) {
                EffectResult::Item(x) => this.flatten.set(Some(x)),
                EffectResult::Failure(_) => unreachable!(),
                EffectResult::Done(x) => return EffectResult::Done(x.into_max()),
                EffectResult::Pending(x) => return EffectResult::Pending(x.into_max()),
            }
        }
    }
}

pin_project_lite::pin_project!(
    /// Produced by the [`flatten_no_error()`](super::EffectiveExt::flatten_no_error) method
    pub struct FlattenNoError<E>
    where
        E: Effective,
    {
        #[pin]
        pub(super) inner: E,
        #[pin]
        pub(super) flatten: Option<E::Item>,
    }
);

impl<E> Effective for FlattenNoError<E>
where
    E: Effective,
    E::Item: Effective<Failure = Infallible>,
    E::Produces: ProducesMultipleWith<<E::Item as Effective>::Produces>,
    E::Async: IsAsyncWith<<E::Item as Effective>::Async>,
{
    type Item = <E::Item as Effective>::Item;
    type Produces =
        <E::Produces as ProducesMultipleWith<<E::Item as Effective>::Produces>>::Produces;
    type Async = <E::Async as IsAsyncWith<<E::Item as Effective>::Async>>::IsAsync;
    type Failure = E::Failure;

    fn poll_effect(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> EffectResult<Self::Item, Self::Failure, Self::Produces, Self::Async> {
        let mut this = self.project();
        loop {
            if let Some(flatten) = this.flatten.as_mut().as_pin_mut() {
                match flatten.poll_effect(cx) {
                    EffectResult::Done(_) => this.flatten.set(None),
                    EffectResult::Failure(_) => unreachable!(),
                    EffectResult::Item(x) => {
                        if !<<E::Item as Effective>::Produces as Produces>::MULTIPLE {
                            this.flatten.set(None);
                        }
                        return EffectResult::Item(x);
                    }
                    EffectResult::Pending(x) => {
                        return EffectResult::Pending(<E::Async as IsAsyncWith<
                            <E::Item as Effective>::Async,
                        >>::from_rhs(x))
                    }
                }
            }

            match this.inner.as_mut().poll_effect(cx) {
                EffectResult::Item(x) => this.flatten.set(Some(x)),
                EffectResult::Failure(x) => return EffectResult::Failure(x),
                EffectResult::Done(x) => return EffectResult::Done(x.into_max()),
                EffectResult::Pending(x) => return EffectResult::Pending(x.into_max()),
            }
        }
    }
}

//! Where common [`Effective`](crate::Effective) wrapper constructors live

mod fallible;
mod from_fn;
mod future;
mod iterator;
mod once;
mod unfold;

pub use self::{
    fallible::{fallible, FromFallible},
    from_fn::{from_fn, from_fn_once, FromFn, FromFnOnce},
    future::{future, FromFuture},
    iterator::{iterator, FromIterator},
    once::{once, Once},
    unfold::{unfold, Unfold},
};

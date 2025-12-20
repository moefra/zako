use bitcode::{Decode, Encode};
use std::fmt::Debug;
use std::hash::Hash;
use zako_digest::hash::XXHash3;

pub trait Persistent<C>: Sized {
    type Persisted: Encode + for<'a> Decode<'a> + Send + Sync;

    fn to_persisted(&self, ctx: &C) -> Option<Self::Persisted>;
    fn from_persisted(p: Self::Persisted, ctx: &C) -> Option<Self>;
}

pub trait NodeKey<C>: Clone + Debug + Eq + Hash + Send + Sync + 'static + Persistent<C> {}

pub trait NodeValue<C>: Debug + Send + Sync + 'static + Persistent<C> {}

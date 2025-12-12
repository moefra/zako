use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::hash::Hash;
use zako_digest::hash::XXHash3;

pub trait Persistent<C> {
    type Persisted: Serialize + DeserializeOwned + Send + Sync;

    fn to_persisted(&self, ctx: &C) -> Self::Persisted;
    fn from_persisted(p: Self::Persisted, ctx: &C) -> Self;
}

pub trait NodeKey<C>:
    Clone + Debug + Eq + Hash + Send + Sync + 'static + XXHash3 + Persistent<C>
{
}

pub trait NodeValue<C>: Debug + Send + Sync + 'static + XXHash3 + Persistent<C> {}

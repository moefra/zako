use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::hash::Hash;

use zako_digest::hash::XXHash3;

pub trait NodeKey:
    Clone + Debug + Eq + Hash + Send + Sync + 'static + Serialize + DeserializeOwned
{
}

impl<T> NodeKey for T where
    T: Clone + Debug + Eq + Hash + Send + Sync + 'static + Serialize + DeserializeOwned
{
}

pub trait NodeValue: Debug + XXHash3 + Send + Sync + 'static {}

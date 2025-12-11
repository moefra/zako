use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::hash::Hash;

pub trait NodeKey:
    Clone + Debug + Eq + Hash + Send + Sync + 'static + Serialize + DeserializeOwned
{
}

impl<T> NodeKey for T where
    T: Clone + Debug + Eq + Hash + Send + Sync + 'static + Serialize + DeserializeOwned
{
}

pub trait NodeValue: Debug + Send + Sync + 'static {}

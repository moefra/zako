use rkyv::{Archive, Deserialize, Serialize};
use std::{fmt::Display, sync::Arc};

use crate::{
    error::HoneError,
    node::{NodeKey, NodeValue, Persistent, SafeNodeKey, SafeNodeValue},
};

pub type Hash = zako_digest::blake3_hash::Hash;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Archive)]
pub struct HashPair {
    output_hash: Hash,
    input_hash: Hash,
}

impl HashPair {
    pub fn new(output_hash: Hash, input_hash: Hash) -> Self {
        Self {
            output_hash,
            input_hash,
        }
    }

    pub fn output_hash(&self) -> &Hash {
        &self.output_hash
    }

    pub fn input_hash(&self) -> &Hash {
        &self.input_hash
    }
}

#[derive(Debug)]
pub struct NodeData<C, V: NodeValue> {
    value: Arc<V>,
    hash_pair: HashPair,
    _marker: std::marker::PhantomData<C>,
}

impl<C, V: NodeValue> Clone for NodeData<C, V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            hash_pair: self.hash_pair.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C, V: NodeValue> NodeData<C, V> {
    pub fn new(hash_pair: HashPair, value: Arc<V>) -> Self {
        Self {
            value,
            hash_pair,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn value(&self) -> &Arc<V> {
        &self.value
    }

    pub fn into_value(self) -> Arc<V> {
        self.value
    }

    pub fn hash_pair(&self) -> &HashPair {
        &self.hash_pair
    }
}

#[derive(Debug)]
pub enum NodeStatus<C, V: NodeValue> {
    Computing(Arc<tokio::sync::Notify>),
    Verified(NodeData<C, V>),
    Dirty(NodeData<C, V>),
    Failed(Arc<HoneError>),
    Unreachable(std::marker::PhantomData<C>),
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub enum NodeStatusCode {
    Unreachable = 0,
    Computing = 1,
    Verified = 2,
    Dirty = 3,
    Failed = 4,
}
impl TryFrom<u8> for NodeStatusCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        match value {
            0 => Ok(NodeStatusCode::Unreachable),
            1 => Ok(NodeStatusCode::Computing),
            2 => Ok(NodeStatusCode::Verified),
            3 => Ok(NodeStatusCode::Dirty),
            4 => Ok(NodeStatusCode::Failed),
            _ => Err(()),
        }
    }
}
pub fn get_node_status_code<C, V: NodeValue>(status: &NodeStatus<C, V>) -> u8 {
    match status {
        NodeStatus::Computing(_) => NodeStatusCode::Computing as u8,
        NodeStatus::Verified(_) => NodeStatusCode::Verified as u8,
        NodeStatus::Dirty(_) => NodeStatusCode::Dirty as u8,
        NodeStatus::Failed(_) => NodeStatusCode::Failed as u8,
        NodeStatus::Unreachable(_) => NodeStatusCode::Unreachable as u8,
    }
}

impl<C, V: NodeValue> Clone for NodeStatus<C, V> {
    fn clone(&self) -> Self {
        match self {
            NodeStatus::Computing(notify) => NodeStatus::Computing(notify.clone()),
            NodeStatus::Verified(data) => NodeStatus::Verified(data.clone()),
            NodeStatus::Dirty(data) => NodeStatus::Dirty(data.clone()),
            NodeStatus::Failed(err) => NodeStatus::Failed(err.clone()),
            NodeStatus::Unreachable(_) => NodeStatus::Unreachable(std::marker::PhantomData),
        }
    }
}

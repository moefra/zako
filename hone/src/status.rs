use bitcode::{Decode, Encode};
use phf::phf_map;
use std::{fmt::Display, sync::Arc};

use crate::{
    error::HoneError,
    node::{NodeValue, Persistent},
};

#[derive(Debug)]
pub struct NodeData<C, V: NodeValue<C>> {
    value: Arc<V>,
    output_xxhash3: u128,
    input_xxhash3: u128,
    _marker: std::marker::PhantomData<C>,
}

impl<C, V: NodeValue<C>> Persistent<C> for NodeData<C, V>
where
    V: NodeValue<C>,
{
    type Persisted = (V::Persisted, u128, u128);

    fn to_persisted(&self, ctx: &C) -> Option<Self::Persisted> {
        Some((
            self.value.to_persisted(ctx)?,
            self.output_xxhash3,
            self.input_xxhash3,
        ))
    }

    fn from_persisted(p: Self::Persisted, ctx: &C) -> Option<Self> {
        Some(Self {
            value: Arc::new(V::from_persisted(p.0, ctx)?),
            output_xxhash3: p.1,
            input_xxhash3: p.2,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<C, V: NodeValue<C>> Clone for NodeData<C, V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            output_xxhash3: self.output_xxhash3,
            input_xxhash3: self.input_xxhash3,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<C, V: NodeValue<C>> NodeData<C, V> {
    pub fn new(value: Arc<V>, output_xxhash3: u128, input_xxhash3: u128) -> Self {
        Self {
            value,
            output_xxhash3,
            input_xxhash3,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn value(&self) -> &Arc<V> {
        &self.value
    }

    pub fn into_value(self) -> Arc<V> {
        self.value
    }

    pub fn input_xxhash3(&self) -> u128 {
        self.input_xxhash3
    }

    pub fn output_xxhash3(&self) -> u128 {
        self.output_xxhash3
    }
}

#[derive(Debug)]
pub enum NodeStatus<C, V: NodeValue<C>> {
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
pub fn get_node_status_code<C, V: NodeValue<C>>(status: &NodeStatus<C, V>) -> u8 {
    match status {
        NodeStatus::Computing(_) => NodeStatusCode::Computing as u8,
        NodeStatus::Verified(_) => NodeStatusCode::Verified as u8,
        NodeStatus::Dirty(_) => NodeStatusCode::Dirty as u8,
        NodeStatus::Failed(_) => NodeStatusCode::Failed as u8,
        NodeStatus::Unreachable(_) => NodeStatusCode::Unreachable as u8,
    }
}

impl<C, V: NodeValue<C>> Clone for NodeStatus<C, V> {
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

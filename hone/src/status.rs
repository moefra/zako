use std::sync::Arc;

use crate::{error::HoneError, node::NodeValue};

#[derive(Debug)]
pub struct NodeData<V: NodeValue> {
    value: Arc<V>,
    output_xxhash3: u128,
    input_xxhash3: u128,
}

impl<V: NodeValue> Clone for NodeData<V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            output_xxhash3: self.output_xxhash3,
            input_xxhash3: self.input_xxhash3,
        }
    }
}

impl<V: NodeValue> NodeData<V> {
    pub fn new(value: Arc<V>, output_xxhash3: u128, input_xxhash3: u128) -> Self {
        Self {
            value,
            output_xxhash3,
            input_xxhash3,
        }
    }

    pub fn value(&self) -> &Arc<V> {
        &self.value
    }

    pub fn into_value(self) -> Arc<V> {
        self.value
    }

    pub fn output_xxhash3(&self) -> u128 {
        self.output_xxhash3
    }
}

#[derive(Debug)]
pub enum NodeStatus<V: NodeValue> {
    Computing(Arc<tokio::sync::Notify>),
    Verified(NodeData<V>),
    Dirty(NodeData<V>),
    Failed(Arc<HoneError>),
}

impl<V: NodeValue> Clone for NodeStatus<V> {
    fn clone(&self) -> Self {
        match self {
            NodeStatus::Computing(notify) => NodeStatus::Computing(notify.clone()),
            NodeStatus::Verified(data) => NodeStatus::Verified(data.clone()),
            NodeStatus::Dirty(data) => NodeStatus::Dirty(data.clone()),
            NodeStatus::Failed(err) => NodeStatus::Failed(err.clone()),
        }
    }
}

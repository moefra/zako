use std::sync::Arc;

use crate::{error::HoneError, node::NodeValue};

#[derive(Debug)]
pub struct NodeData<C, V: NodeValue<C>> {
    value: Arc<V>,
    output_xxhash3: u128,
    input_xxhash3: u128,
    _marker: std::marker::PhantomData<C>,
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

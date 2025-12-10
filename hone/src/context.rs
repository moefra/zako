use std::hash::Hash;
use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;

use crate::SharedHoneResult;
use crate::status::NodeData;
use crate::{
    FastMap, FastSet, HoneResult,
    engine::Engine,
    node::{NodeKey, NodeValue},
    status::NodeStatus,
};

#[async_trait]
pub trait Computer<K: NodeKey, V: NodeValue>: Send + Sync + Debug {
    async fn compute(&self, ctx: &Context<K, V>) -> HoneResult<NodeData<V>>;
}

#[derive(Debug)]
pub struct Context<'a, K: NodeKey, V: NodeValue> {
    pub engine: &'a Engine<K, V>,
    pub caller: Option<K>,
    pub this: &'a K,
    pub stack: im::Vector<K>,
    pub old_data: Option<NodeData<V>>,
}

impl<'a, K: NodeKey, V: NodeValue> Context<'a, K, V> {
    /// 请求一个依赖项
    /// 1. 将 (caller -> key) 边写入依赖图
    /// 2. 异步等待 key 计算完成
    /// 3. 返回结果
    pub async fn request(&mut self, key: K) -> SharedHoneResult<NodeData<V>> {
        if self.stack.contains(&key) {
            return Err(Arc::new(crate::error::HoneError::CycleDetected {
                caller: self
                    .stack
                    .iter()
                    .cloned()
                    .chain(std::iter::once(key))
                    .map(|item| format!("{:?}", item))
                    .collect(),
                current: format!("{:?}", self.this.clone()),
            }));
        }

        let mut stack = self.stack.clone();
        stack.push_back(key.clone());

        // 1. 动态注册依赖
        self.engine
            .get_dependency_graph()
            .add_child(self.this.clone(), key.clone());
        self.engine
            .get_dependency_graph()
            .add_parent(key.clone(), self.this.clone());

        // 2. 触发获取（如果 key 还没算，会在这里触发计算；如果正在算，会等待）
        self.engine.get(key, Some(self.this.clone()), stack).await
    }
}

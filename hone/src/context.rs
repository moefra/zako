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
pub trait Computer<C, K: NodeKey<C>, V: NodeValue<C>>: Send + Sync + Debug {
    async fn compute<'c>(&self, ctx: &'c Context<C, K, V>) -> HoneResult<NodeData<C, V>>;
}

#[derive(Debug)]
pub struct Context<'c, C, K: NodeKey<C>, V: NodeValue<C>> {
    engine: &'c Engine<C, K, V>,
    caller: Option<K>,
    this: &'c K,
    stack: im::Vector<K>,
    old_data: Option<NodeData<C, V>>,
    context: &'c C,
}

impl<'c, C, K: NodeKey<C>, V: NodeValue<C>> Context<'c, C, K, V> {
    pub fn new(
        engine: &'c Engine<C, K, V>,
        caller: Option<K>,
        this: &'c K,
        stack: im::Vector<K>,
        old_data: Option<NodeData<C, V>>,
        context: &'c C,
    ) -> Self {
        Self {
            engine,
            caller,
            this,
            stack,
            old_data,
            context,
        }
    }

    pub fn engine(&'c self) -> &'c Engine<C, K, V> {
        self.engine
    }

    pub fn caller(&self) -> Option<&K> {
        self.caller.as_ref()
    }

    pub fn this(&'c self) -> &'c K {
        self.this
    }

    pub fn old_data(&'c self) -> Option<&'c NodeData<C, V>> {
        self.old_data.as_ref()
    }

    pub fn context(&'c self) -> &'c C {
        self.context
    }

    /// 请求一个依赖项
    /// 1. 将 (caller -> key) 边写入依赖图
    /// 2. 异步等待 key 计算完成
    /// 3. 返回结果
    pub async fn request(&self, key: K) -> SharedHoneResult<NodeData<C, V>> {
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
        // TODO: 优化这里的锁粒度
        // TODO: Does this operate correctly in concurrent scenarios?
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

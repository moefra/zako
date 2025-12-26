use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;

use crate::SharedHoneResult;
use crate::error::HoneError;
use crate::status::NodeData;
use crate::{
    HoneResult,
    engine::Engine,
    node::{NodeKey, NodeValue},
};

#[async_trait]
pub trait Computer<C, K, V>: Send + Sync + Debug
where
    K: NodeKey,
    V: NodeValue,
{
    async fn compute<'c>(&self, ctx: &'c Context<C, K, V>) -> HoneResult<NodeData<C, V>>;
}

#[derive(Debug)]
pub struct Context<'c, C, K: NodeKey, V: NodeValue> {
    engine: &'c Engine<C, K, V>,
    caller: Option<K>,
    this: &'c K,
    stack: im::Vector<K>,
    old_data: Option<NodeData<C, V>>,
    context: &'c C,
    cancel_token: zako_cancel::CancelToken,
}

impl<'c, C, K: NodeKey, V: NodeValue> Context<'c, C, K, V> {
    pub fn new(
        engine: &'c Engine<C, K, V>,
        caller: Option<K>,
        this: &'c K,
        stack: im::Vector<K>,
        old_data: Option<NodeData<C, V>>,
        context: &'c C,
        cancel_token: zako_cancel::CancelToken,
    ) -> Self {
        Self {
            engine,
            caller,
            this,
            stack,
            old_data,
            context,
            cancel_token,
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

    pub fn cancel_token(&self) -> zako_cancel::CancelToken {
        self.cancel_token.clone()
    }

    pub async fn request_with_context(
        &self,
        key: K,
        context: &C,
    ) -> SharedHoneResult<NodeData<C, V>> {
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

        // Check cancel token here when we done prepare operation
        if self.cancel_token.is_cancelled() {
            return Err(Arc::new(HoneError::Canceled {
                reason: self.cancel_token.reason().clone(),
            }));
        }

        self.engine
            .get(
                key,
                Some(self.this.clone()),
                stack,
                self.cancel_token.clone(),
                context,
            )
            .await
    }

    /// 请求一个依赖项
    /// 1. 将 (caller -> key) 边写入依赖图
    /// 2. 异步等待 key 计算完成
    /// 3. 返回结果
    pub async fn request(&self, key: K) -> SharedHoneResult<NodeData<C, V>> {
        self.request_with_context(key, self.context).await
    }
}

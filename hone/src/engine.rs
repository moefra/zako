use crate::dependency::DependencyGraph;
use crate::node::Persistent;
use crate::status::{NodeStatusCode, get_node_status_code};
use crate::{FastMap, HoneResult, SharedHoneResult, context::Context, status::NodeData};
use crate::{FastSet, TABLE_NODES};
use ahash::{AHashMap, HashSet, HashSetExt};
use dashmap::DashMap;
use dashmap::Entry::{Occupied, Vacant};
use eyre::Error;
use futures::StreamExt;
use redb::{ReadableDatabase, ReadableTable};
use redb::{TableError, TransactionError};
use std::ops::Not;
use std::rc::Rc;
use std::sync::Arc;
use tracing::event;

use crate::{
    context::Computer,
    error::HoneError,
    node::{NodeKey, NodeValue},
    status::{self, NodeStatus},
};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Redb Database error: {0}")]
    DatabaseError(#[from] TransactionError),
    #[error("Redb Table error: {0}")]
    TableError(#[from] TableError),
    #[error("Redb Commit error: {0}")]
    CommitError(#[from] redb::CommitError),
    #[error("Redb Storage error: {0}")]
    StorageError(#[from] redb::StorageError),
    #[error("Other error: {0}")]
    Other(#[from] eyre::Report),
    #[error("Invalid pollute action for node `{0}`: {1}")]
    InvalidPolluteAction(String, String),
}

#[derive(Debug)]
pub struct Engine<C, K: NodeKey<C>, V: NodeValue<C>> {
    status_map: DashMap<K, NodeStatus<C, V>>,
    computer: Arc<dyn Computer<C, K, V>>,
    dependency_graph: Arc<DependencyGraph<C, K>>,
    database: Arc<redb::Database>,
    context: Arc<C>,
}

#[derive(Debug, Clone)]
pub struct ResolveOptions {
    pub buffered_count: usize,
    pub keep_going: bool,
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            buffered_count: 10,
            keep_going: true,
        }
    }
}

impl<C, K: NodeKey<C>, V: NodeValue<C>> Engine<C, K, V> {
    pub fn new(
        computer: Arc<dyn Computer<C, K, V>>,
        database: Arc<redb::Database>,
        context: Arc<C>,
    ) -> Result<Self, EngineError> {
        let this = Self {
            status_map: DashMap::new(),
            computer: computer,
            dependency_graph: Arc::new(DependencyGraph::new()),
            database,
            context,
        };
        this.fill_from_db()?;
        Ok(this)
    }

    fn fill_from_db(&self) -> Result<(), EngineError> {
        let txn = self.database.begin_read()?;
        let table = txn.open_table(TABLE_NODES)?;
        for entry in table.iter()? {
            let (key_bytes, value_bytes) = entry?;

            let key_bytes = key_bytes.value();

            if key_bytes.is_empty() {
                tracing::event!(
                    tracing::Level::ERROR,
                    "The key bytes is empty. We expect it has a one-byte key node status code. Skip",
                );
                continue;
            }

            let code: NodeStatusCode = match key_bytes[0].try_into() {
                Ok(code) => code,
                Err(_) => {
                    tracing::event!(
                        tracing::Level::ERROR,
                        "Invalid node status code `{}`. Skip",
                        key_bytes[0]
                    );
                    continue;
                }
            };
            match code {
                NodeStatusCode::Verified => {
                    let value_bytes = value_bytes.value();

                    // read the output_xxhash3 and input_xxhash3 from the value_bytes
                    let output_xxhash3 = u128::from_le_bytes(match value_bytes[1..17].try_into() {
                        Ok(ok) => ok,
                        Err(err) => {
                            tracing::event!(
                                tracing::Level::ERROR,
                                "Failed to call from_le_bytes() when decode output_xxhash3 `{:?}`. Skip",
                                err
                            );
                            continue;
                        }
                    });
                    let input_xxhash3 = u128::from_le_bytes(match value_bytes[17..33].try_into() {
                        Ok(ok) => ok,
                        Err(err) => {
                            tracing::event!(
                                tracing::Level::ERROR,
                                "Failed to call from_le_bytes() when decode input_xxhash3 `{:?}`. Skip",
                                err
                            );
                            continue;
                        }
                    });
                    let value_bytes = &value_bytes[33..];

                    let node_data = match bitcode::decode::<V::Persisted>(value_bytes) {
                        Ok(ok) => ok,
                        Err(err) => {
                            tracing::event!(
                                tracing::Level::ERROR,
                                "Failed to call bitcode::decode() when decode node data `{:?}`. Skip",
                                err
                            );
                            continue;
                        }
                    };

                    let node_key = match bitcode::decode::<K::Persisted>(
                        // skip the first byte, which is the node status code
                        &key_bytes[1..],
                    ) {
                        Ok(ok) => ok,
                        Err(err) => {
                            tracing::event!(
                                tracing::Level::ERROR,
                                "Failed to call bitcode::decode() when decode node key `{:?}`. Skip",
                                err
                            );
                            continue;
                        }
                    };

                    let node_key = K::from_persisted(node_key, self.context.clone().as_ref());
                    let node_data = V::from_persisted(node_data, self.context.clone().as_ref());

                    if let Some(node_data) = node_data
                        && let Some(node_key) = node_key
                    {
                        let node_status = NodeStatus::Verified(NodeData::new(
                            input_xxhash3,
                            output_xxhash3,
                            Arc::new(node_data),
                        ));
                        self.insert(node_key, node_status, None, None);
                    } else {
                        tracing::event!(
                            tracing::Level::ERROR,
                            "Failed to decode node key or data. Skip",
                        );
                        continue;
                    }
                }
                // TODO: Support Dirty and Failure
                // Issue URL: https://github.com/moefra/zako/issues/9
                _ => {
                    event!(
                        tracing::Level::ERROR,
                        "Unsupported node status code `{}`. Skip",
                        code as u8
                    );
                }
            }
        }
        Ok(())
    }

    pub fn peek_status(&self, key: &K) -> Option<NodeStatus<C, V>> {
        self.status_map.get(key).map(|entry| (*entry).clone())
    }

    pub fn insert(
        &self,
        key: K,
        status: NodeStatus<C, V>,
        parent: Option<FastSet<K>>,
        child: Option<FastSet<K>>,
    ) {
        self.status_map.insert(key.clone(), status);
        if let Some(parents) = parent {
            self.dependency_graph
                .add_parents(key.clone(), parents.into_iter());
        }
        if let Some(children) = child {
            self.dependency_graph
                .add_children(key.clone(), children.into_iter());
        }
    }

    pub fn pollute(&self, key: K, status: NodeStatus<C, V>) -> Result<(), EngineError> {
        if self.status_map.contains_key(&key).not() {
            return Err(EngineError::InvalidPolluteAction(
                format!("{:?}", key),
                "Key not found".to_string(),
            ));
        }
        if let NodeStatus::Dirty(_) = status {
            self.status_map.insert(key.clone(), status);
        }
        return Err(EngineError::InvalidPolluteAction(
            format!("{:?}", key),
            "Only Dirty status can be used to pollute".to_string(),
        ));
    }

    /// Write node to the database, persisting only Verified and Dirty nodes.
    ///
    /// [NodeStatus::Computing] and [NodeStatus::Failed] are not persisted.
    ///
    /// All written node will seems as dirty.
    ///
    /// It will also skip nodes key or value that return None when calling [Persistent::to_persisted].
    ///
    /// TODO: Implement negative cache.
    //Issue URL: https://github.com/moefra/zako/issues/8
    /// Issue URL: https://github.com/moefra/zako/issues/7
    pub fn write(&self) -> Result<(), EngineError> {
        let context = self.context.clone();
        let txn = self.database.begin_write()?;
        {
            let mut table = txn.open_table(TABLE_NODES)?;

            for entry in self.status_map.iter() {
                let value_bytes = match entry.value() {
                    NodeStatus::Verified(data) => match &data.to_persisted(context.as_ref()) {
                        Some(persisted) => (
                            bitcode::encode(persisted),
                            data.output_xxhash3(),
                            data.input_xxhash3(),
                        ),
                        None => continue,
                    },
                    NodeStatus::Dirty(data) => match &data.to_persisted(context.as_ref()) {
                        Some(persisted) => (
                            bitcode::encode(persisted),
                            data.output_xxhash3(),
                            data.input_xxhash3(),
                        ),
                        None => continue,
                    },
                    NodeStatus::Failed(err) => (
                        format!("Failed node `{:?}`: {:?}", entry.key(), err)
                            .as_bytes()
                            .to_vec(),
                        0,
                        0,
                    ),
                    _ => {
                        continue;
                    }
                };

                let value_bytes = vec![
                    value_bytes.0.as_slice(),
                    &value_bytes.1.to_le_bytes(),
                    &value_bytes.2.to_le_bytes(),
                ]
                .concat();

                let mut key_bytes = vec![get_node_status_code(entry.value()) as u8];
                key_bytes.extend(bitcode::encode(
                    &match entry.key().to_persisted(context.as_ref()) {
                        Some(k) => k,
                        None => continue,
                    },
                ));

                table.insert(key_bytes.as_slice(), value_bytes.as_slice())?;
            }
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_computer(&self) -> Arc<dyn Computer<C, K, V>> {
        self.computer.clone()
    }

    pub fn get_dependency_graph(&self) -> &DependencyGraph<C, K> {
        &self.dependency_graph
    }

    pub async fn get(
        &self,
        key: K,
        caller: Option<K>,
        stack: im::Vector<K>,
        cancel_token: zako_cancel::CancelToken,
    ) -> SharedHoneResult<NodeData<C, V>> {
        let mut result: Option<SharedHoneResult<NodeData<C, V>>> = None;
        let context = self.context.clone();

        loop {
            let notify = Arc::new(tokio::sync::Notify::new());
            let old = {
                let entry = self.status_map.entry(key.clone());

                // double check
                match entry {
                    Occupied(mut occupied_entry) => {
                        let entry_ref = occupied_entry.get();

                        match entry_ref {
                            NodeStatus::Verified(data) => {
                                result = Some(Ok(data.clone()));
                                drop(occupied_entry); // 释放锁
                                break;
                            }
                            NodeStatus::Computing(existing_notify) => {
                                // 其他任务正在计算，等待其完成
                                let existing_notify = existing_notify.clone();
                                drop(occupied_entry); // 释放锁
                                existing_notify.notified().await;
                                continue; // 重试获取结果
                            }
                            NodeStatus::Dirty(data) => {
                                let old = Some(data.clone());
                                occupied_entry.insert(NodeStatus::Computing(notify.clone()));
                                // 新任务，注册计算
                                // 同时初始化依赖图中的节点
                                self.dependency_graph
                                    .clear_children_dependency_of(key.clone());
                                // 需要重新计算，继续往下走
                                old
                            }
                            NodeStatus::Failed(err) => {
                                result = Some(Err(err.clone()));
                                drop(occupied_entry); // 释放锁
                                break;
                            }
                            NodeStatus::Unreachable(_) => {
                                let err = Arc::new(HoneError::UnexpectedError(
                                    "Node is unreachable".to_string(),
                                ));
                                result = Some(Err(err));
                                drop(occupied_entry); // 释放锁
                                break;
                            }
                        }
                    }
                    Vacant(entry) => {
                        // 抢到了！将状态设为 Computing
                        entry.insert(NodeStatus::Computing(notify.clone()));
                        // 新任务，注册计算
                        // 同时初始化依赖图中的节点
                        self.dependency_graph
                            .clear_children_dependency_of(key.clone());

                        None
                    }
                }
            }; // 锁在这里释放

            // check cancel token here
            if cancel_token.is_cancelled() {
                return Err(Arc::new(HoneError::Canceled {
                    reason: cancel_token.reason().clone(),
                }));
            }

            // --- 步骤 5: 执行计算 (无锁状态！) ---
            // 创建一个新的 Context，标记当前节点为 caller
            let ctx: Context<'_, C, K, V> = Context::new(
                self,
                caller,
                &key,
                stack,
                old,
                context.as_ref(),
                cancel_token.clone(),
            );

            // 真正的运行用户逻辑
            let computed = self.computer.compute(&ctx).await;

            // --- 步骤 6: 提交结果 ---
            result = Some(computed.map_err(|err: HoneError| Arc::new(err)));
            break;
        }

        return result.ok_or_else(|| {
            Arc::new(HoneError::UnexpectedError(
                "Engine get: unexpected missing result".to_string(),
            ))
        })?;
    }

    pub async fn resolve_inner(
        &self,
        key: K,
        caller: Option<K>,
        search_stack: &mut im::Vector<K>,
        cancel_token: zako_cancel::CancelToken,
        options: ResolveOptions,
    ) -> SharedHoneResult<NodeData<C, V>> {
        // check cancel token here
        if cancel_token.is_cancelled() {
            return Err(Arc::new(HoneError::Canceled {
                reason: cancel_token.reason().clone(),
            }));
        }

        // check circular dependency
        if search_stack.contains(&key) {
            return Err(Arc::new(HoneError::CycleDetected {
                caller: Vec::with_capacity(search_stack.len()),
                current: format!("{:?}", key),
            }));
        }

        search_stack.push_back(key.clone());

        // resolve children
        match self.dependency_graph.get_children(key.clone()) {
            Occupied(children_entry) => {
                let locked = children_entry.get();
                let children: Vec<K> = locked.iter().map(|arc| arc.clone()).collect();
                drop(children_entry); // 释放锁

                let mut stream = futures::stream::iter(children)
                    .map(|child| {
                        let engine_ref = self;
                        let mut search_stack = search_stack.clone();
                        let caller = Some(key.clone());
                        let cancel_token = cancel_token.clone();
                        let options = options.clone();
                        return async move {
                            // check cancel token here
                            if cancel_token.is_cancelled() {
                                return Err(Arc::new(HoneError::Canceled {
                                    reason: cancel_token.reason().clone(),
                                }));
                            }

                            // resolve child
                            match engine_ref
                                .resolve_inner(
                                    child.clone(),
                                    caller,
                                    &mut search_stack,
                                    cancel_token,
                                    options,
                                )
                                .await
                            {
                                Ok(_) => Ok(()),
                                Err(e) => {
                                    if let HoneError::CycleDetected { caller, current } = &*e {
                                        let mut caller = caller.clone();
                                        caller.push(format!("{:?}", current));
                                        Err(e)
                                    } else {
                                        Err(e)
                                    }
                                }
                            }
                        };
                    })
                    .buffer_unordered(options.buffered_count);

                let mut errors = Vec::new();

                while let Some(result) = stream.next().await {
                    // 4. 检查取消 (运行时检查)
                    // 很有可能在等待子任务时，外部触发了取消
                    if cancel_token.is_cancelled() {
                        return Err(Arc::new(HoneError::Canceled {
                            reason: cancel_token.reason().clone(),
                        }));
                    }

                    match result {
                        Ok(_) => continue, // 成功，继续下一个
                        Err(e) => {
                            // 🔥 Fail-Fast 触发点！
                            // 直接 return Err。
                            // `stream` 变量会被 Drop。
                            // stream 内部正在跑的其他 Future 也会被 Drop (即被取消)。

                            if let HoneError::CycleDetected { .. } = &*e {
                                return Err(e);
                            }

                            if options.keep_going {
                                errors.push(e);
                            } else {
                                return Err(e);
                            }
                        }
                    }
                }
            }
            _ => {}
        };

        let result = self
            .get(
                key.clone(),
                caller,
                search_stack.clone(),
                cancel_token.clone(),
            )
            .await;

        search_stack.pop_back();

        result
    }

    pub async fn resolve(
        &self,
        key: K,
        cancel_token: zako_cancel::CancelToken,
        options: ResolveOptions,
    ) -> SharedHoneResult<NodeData<C, V>> {
        let mut search_stack = im::Vector::<K>::new();
        self.resolve_inner(key, None, &mut search_stack, cancel_token.clone(), options)
            .await
    }
}

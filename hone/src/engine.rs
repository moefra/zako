use crate::dependency::DependencyGraph;
use crate::node::Persistent;
use crate::{FastMap, HoneResult, SharedHoneResult, context::Context, status::NodeData};
use crate::{FastSet, TABLE_NODES};
use ahash::{AHashMap, HashSet, HashSetExt};
use dashmap::DashMap;
use dashmap::Entry::{Occupied, Vacant};
use eyre::Error;
use futures::StreamExt;
use redb::{TableError, TransactionError};
use std::ops::Not;
use std::rc::Rc;
use std::sync::Arc;

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

impl<C, K: NodeKey<C>, V: NodeValue<C>> Engine<C, K, V> {
    pub fn new(
        computer: Arc<dyn Computer<C, K, V>>,
        database: Arc<redb::Database>,
        context: Arc<C>,
    ) -> Self {
        Self {
            status_map: DashMap::new(),
            computer: computer,
            dependency_graph: Arc::new(DependencyGraph::new()),
            database,
            context,
        }
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
    pub fn write(&self, ctx: &C) -> Result<(), EngineError> {
        let txn = self.database.begin_write()?;
        {
            let mut table = txn.open_table(TABLE_NODES)?;

            for entry in self.status_map.iter() {
                let key_bytes = bitcode::encode(&match entry.key().to_persisted(ctx) {
                    Some(k) => k,
                    None => continue,
                });

                let value_bytes = match entry.value() {
                    NodeStatus::Verified(data) => match &data.to_persisted(ctx) {
                        Some(persisted) => bitcode::encode(persisted),
                        None => continue,
                    },
                    NodeStatus::Dirty(data) => match &data.to_persisted(ctx) {
                        Some(persisted) => bitcode::encode(persisted),
                        None => continue,
                    },
                    _ => {
                        continue;
                    }
                };
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
    ) -> SharedHoneResult<NodeData<C, V>> {
        let mut result: Option<SharedHoneResult<NodeData<C, V>>> = None;

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

            // --- 步骤 5: 执行计算 (无锁状态！) ---
            // 创建一个新的 Context，标记当前节点为 caller
            let ctx: Context<'_, C, K, V> = Context::new(self, caller, &key, stack, old);

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
        buffered_count: usize,
    ) -> SharedHoneResult<NodeData<C, V>> {
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

                let errors = futures::stream::iter(children)
                    .map(|child| {
                        let engine_ref = self;
                        let mut search_stack = search_stack.clone();
                        let caller = Some(key.clone());
                        return async move {
                            match engine_ref
                                .resolve_inner(child.clone(), caller, &mut search_stack, 1)
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
                    .buffer_unordered(buffered_count)
                    .collect::<Vec<SharedHoneResult<()>>>()
                    .await
                    .iter()
                    .filter_map(|item| item.clone().err())
                    .collect::<Vec<Arc<HoneError>>>();

                if !errors.is_empty() {
                    return Err(Arc::new(HoneError::AggregativeError(errors)));
                }
            }
            _ => {}
        };

        let result = self.get(key.clone(), caller, search_stack.clone()).await;

        search_stack.pop_back();

        result
    }

    pub async fn resolve(&self, key: K, buffered_count: usize) -> SharedHoneResult<NodeData<C, V>> {
        let mut search_stack = im::Vector::<K>::new();
        self.resolve_inner(key, None, &mut search_stack, buffered_count)
            .await
    }
}

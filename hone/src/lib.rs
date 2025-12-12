use std::sync::Arc;

use redb::TableDefinition;

pub mod context;
pub mod dependency;
pub mod engine;
pub mod error;
pub mod node;
pub mod status;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct KeyId(u64);

/// The result of iteration of this map is not ordered.
///
/// Please do not rely on any specific order.
pub type FastMap<K, V> = ::dashmap::DashMap<K, V, ::ahash::RandomState>;

/// The result of iteration of this map is not ordered.
///
/// Please do not rely on any specific order.
pub type FastSet<K> = ::dashmap::DashSet<K, ::ahash::RandomState>;

pub type HoneResult<T> = Result<T, error::HoneError>;

pub type SharedHoneResult<T> = Result<T, Arc<error::HoneError>>;

const TABLE_NODES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("hone_v1_nodes");

const TABLE_PARENTS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("hone_v1_parents");

const TABLE_CHILDREN: TableDefinition<&[u8], &[u8]> = TableDefinition::new("hone_v1_children");

use std::{
    hash::Hash,
    marker::PhantomData,
    num::NonZeroU32,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};

use dashmap::DashMap;
use redb::{ReadableTable, TableDefinition, TableError};
use rkyv::Serialize;
use smol_str::SmolStr;

mod chunk;
mod id_map;
mod key;
mod pool;

pub use key::{Key, U32NonZeroKey};
pub use pool::PoolOptions;

use crate::{id_map::IdMap, pool::Pool};

/// A persistent, thread-safe string interner backed by redb.
///
/// It maintains an in-memory cache for fast lookups and can be committed to a database.
/// The interner uses DashMap for thread-safe in-memory storage and AtomicUsize for ID generation.
///
pub struct PersistentInterner<K: Key> {
    pool: Pool,
    id_map: IdMap,
    next_id: AtomicU64,
    key: PhantomData<K>,
}

pub struct InternerOptions {
    pool_options: PoolOptions,
}

impl<K: Key> PersistentInterner<K> {
    pub fn new(options: InternerOptions) -> Self {
        Self {
            pool: Pool::new(options.pool_options),
            id_map: IdMap::new(),
            next_id: AtomicU64::new(0),
            key: PhantomData,
        }
    }
}

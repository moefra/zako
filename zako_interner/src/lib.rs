use std::{
    hash::Hash,
    marker::PhantomData,
    num::NonZeroU32,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};

use dashmap::DashMap;
use dashmap::Entry::Occupied;
use dashmap::Entry::Vacant;
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
    map: DashMap<u128, K, ahash::RandomState>,
}

pub struct InternerOptions {
    pool_options: PoolOptions,
    map_hasher: ahash::RandomState,
}

impl<K: Key> PersistentInterner<K> {
    pub fn new(options: InternerOptions) -> Self {
        Self {
            pool: Pool::new(options.pool_options),
            id_map: IdMap::new(),
            next_id: AtomicU64::new(0),
            key: PhantomData,
            map: DashMap::with_capacity_and_hasher(1024, options.map_hasher),
        }
    }

    pub fn get_or_intern(&self, string: &str) -> K {
        let hash = xxhash_rust::xxh3::xxh3_64(string.as_bytes());

        match self.map.entry(hash) {
            Occupied(entry) => entry.get().clone(),
            Vacant(entry) => {
                let allocated = self.pool.alloc(string.len() + 8).unwrap();
                let chunk_idx = allocated.0.index;
                let offset = allocated.1 + 8; // 8 bytes for the length of the string

                let data = self.pool.access_mut(chunk_idx, offset, string.len() + 8);

                // write len
                let len_bytes = string.len().to_le_bytes();
                data[..8].copy_from_slice(&len_bytes);
                // write string
                data[8..].copy_from_slice(string.as_bytes());

                // SAFETY: We have uniquely allocated and initialized this memory, and ensure it won't be freed for the program's lifetime.
                let string_ref: &'static str = unsafe {
                    let string_bytes = &data[8..8 + string.len()];
                    std::mem::transmute::<&str, &'static str>(std::str::from_utf8_unchecked(
                        string_bytes,
                    ))
                };

                let id = self.next_id.fetch_add(1, Ordering::AcqRel);
                let id = K::try_from_u64(id).unwrap();

                entry.insert(id);

                self.id_map
                    .register(id.into_u64(), chunk_idx, (offset + 8) as u64);

                id
            }
        }
    }
}

use std::{
    hash::Hash,
    num::NonZeroU32,
    sync::atomic::{AtomicUsize, Ordering},
};

use dashmap::DashMap;
use redb::{ReadableTable, TableDefinition, TableError};
use rkyv::Serialize;
use smol_str::SmolStr;

/// A trait for types that can be used as keys in the interner.
///
/// This is similar to lasso's `Key` trait.
pub trait Key: Copy + Eq + Hash + Send + Sync + 'static {
    /// Returns the [u64] that represents the current key
    fn into_u64(self) -> u64;

    /// Attempts to create a key from a [u64], returning `None` if it fails
    fn try_from_u64(int: u64) -> Option<Self>;
}

/// Why NonZeroU32?
///
/// It can make `Option<InternedString>` take only 4 bytes instead of 8 bytes,
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U32NonZeroKey(NonZeroU32);

impl Key for U32NonZeroKey {
    #[inline]
    fn into_u64(self) -> u64 {
        self.0.get() as u64 - 1
    }

    /// Returns `None` if `int` is greater than `u32::MAX - 1`
    #[inline]
    fn try_from_u64(int: u64) -> Option<Self> {
        if int < u32::MAX as u64 {
            // Safety: The integer is less than the max value and then incremented by one, meaning that
            // is is impossible for a zero to inhabit the NonZeroU32
            unsafe { Some(Self(NonZeroU32::new_unchecked(int as u32 + 1))) }
        } else {
            None
        }
    }
}

/// A persistent, thread-safe string interner backed by redb.
///
/// It maintains an in-memory cache for fast lookups and can be committed to a database.
/// The interner uses DashMap for thread-safe in-memory storage and AtomicUsize for ID generation.
///
/// TODO: Implement a more efficient implementation that uses a byte array to store the string.
//Issue URL: https://github.com/moefra/zako/issues/24
/// Issue URL: https://github.com/moefra/zako/issues/23
pub struct PersistentInterner<K: Key> {
    // String to Key mapping for fast lookups
    str_to_id: DashMap<SmolStr, K, ahash::RandomState>,
    // Key to String mapping for resolution
    id_to_str: DashMap<K, SmolStr, ahash::RandomState>,
    // The number of strings already persisted to the database
    persisted_count: AtomicUsize,
    // The next available ID for interning
    next_id: AtomicUsize,
}

impl<K: Key> PersistentInterner<K> {
    /// Creates a new, empty interner.
    pub fn new() -> Self {
        Self {
            str_to_id: DashMap::with_hasher(ahash::RandomState::default()),
            id_to_str: DashMap::with_hasher(ahash::RandomState::default()),
            persisted_count: AtomicUsize::new(0),
            next_id: AtomicUsize::new(0),
        }
    }

    /// Loads the interner state from a redb database.
    ///
    /// # Arguments
    /// * `txn` - A read transaction from redb.
    /// * `table_def` - The table definition where strings are stored.
    pub fn load(
        txn: &redb::ReadTransaction,
        table_def: TableDefinition<u64, &str>,
    ) -> Result<Self, TableError> {
        let table = txn.open_table(table_def)?;
        let str_to_id = DashMap::with_hasher(ahash::RandomState::default());
        let id_to_str = DashMap::with_hasher(ahash::RandomState::default());
        let mut max_id = 0;
        let mut count = 0;

        let iter = table.iter().map_err(TableError::Storage)?;
        for result in iter {
            let (id, s) = result.map_err(TableError::Storage)?;
            let id_val = id.value();
            let s_val = SmolStr::new(s.value());
            let k = K::try_from_u64(id_val)
                .expect("Interner key space exhausted or corrupted database");

            str_to_id.insert(s_val.clone(), k);
            id_to_str.insert(k, s_val);
            count += 1;

            if id_val as usize >= max_id {
                max_id = id_val as usize + 1;
            }
        }

        Ok(Self {
            str_to_id,
            id_to_str,
            persisted_count: AtomicUsize::new(count),
            next_id: AtomicUsize::new(max_id),
        })
    }

    /// Interns a string and returns its unique key.
    /// If the string is already interned, its existing key is returned.
    ///
    /// This method is thread-safe and can be called from multiple threads simultaneously.
    pub fn get_or_intern(&self, s: &str) -> K {
        if let Some(k) = self.str_to_id.get(s) {
            return *k;
        }

        let s_smol = SmolStr::new(s);
        let mut created_k = None;
        let k = *self.str_to_id.entry(s_smol.clone()).or_insert_with(|| {
            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            let k = K::try_from_u64(id as u64).expect("Interner key space exhausted");
            created_k = Some(k);
            k
        });

        if let Some(k) = created_k {
            self.id_to_str.insert(k, s_smol);
        }

        k
    }

    /// Returns the string associated with the given key, if any.
    pub fn resolve(&self, k: K) -> Option<SmolStr> {
        self.id_to_str.get(&k).map(|s| s.clone())
    }

    /// Commits all newly interned strings to the redb database.
    ///
    /// # Arguments
    /// * `txn` - A write transaction from redb.
    /// * `table_def` - The table definition where strings should be stored.
    pub fn commit(
        &self,
        txn: &redb::WriteTransaction,
        table_def: TableDefinition<u64, &str>,
    ) -> Result<(), TableError> {
        let mut table = txn.open_table(table_def)?;

        let current_next = self.next_id.load(Ordering::SeqCst);
        let mut current_persisted = self.persisted_count.load(Ordering::SeqCst);

        while current_persisted < current_next {
            if let Some(k) = K::try_from_u64(current_persisted as u64) {
                if let Some(s) = self.id_to_str.get(&k) {
                    table
                        .insert(current_persisted as u64, s.as_str())
                        .map_err(TableError::Storage)?;
                }
            }
            current_persisted += 1;
        }

        self.persisted_count
            .store(current_persisted, Ordering::SeqCst);
        Ok(())
    }

    /// Returns the total number of interned strings.
    pub fn len(&self) -> usize {
        self.next_id.load(Ordering::SeqCst)
    }

    /// Returns true if the interner is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K: Key> Default for PersistentInterner<K> {
    fn default() -> Self {
        Self::new()
    }
}

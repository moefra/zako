use std::{
    hash::Hash,
    sync::{
        Arc,
        atomic::{AtomicU32, AtomicUsize, Ordering},
    },
};

use dashmap::{DashMap, Entry};
use redb::TableDefinition;
use smol_str::SmolStr;

type FashMap<K, V> = ::dashmap::DashMap<K, V, ::ahash::RandomState>;
type FastSet<K> = ::dashmap::DashSet<K, ::ahash::RandomState>;

pub trait Key: Copy + Eq + Hash {
    /// Returns the `usize` that represents the current key
    fn into_usize(self) -> usize;

    /// Attempts to create a key from a `usize`, returning `None` if it fails
    fn try_from_usize(int: usize) -> Option<Self>;
}

pub struct PersistentInterner<K: Key> {
    confirmed: FashMap<smol_str::SmolStr, K>,
    re_confirmed: FashMap<K, smol_str::SmolStr>,
    pending: FashMap<smol_str::SmolStr, usize>,
    re_pending: FashMap<usize, smol_str::SmolStr>,
    /// Start from 0
    next_id: AtomicUsize,
}

impl<K: Key> PersistentInterner<K> {
    fn increase_next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn get_or_intern(&self, s: SmolStr) -> K {
        if let Some(id) = self.confirmed.get(s.as_str()) {
            return *id;
        }

        let entry = self.pending.entry(s.clone());

        return K::try_from_usize(match entry {
            Entry::Vacant(v) => {
                let id = self.increase_next_id();
                v.insert(id);
                self.re_pending.insert(id, s);
                id
            }
            Entry::Occupied(o) => *o.get(),
        })
        .unwrap();
    }

    pub fn resolve(&self, k: K) -> SmolStr {
        self.re_confirmed.get(&k).unwrap().clone()
    }

    /// Commit the interner to the database
    ///
    /// # Arguments
    ///
    /// * `txn` - The write transaction to commit to
    /// * `id_to_str_table_definition` - The definition of the table to store the id to string
    /// * `str_to_id_table_definition` - The definition of the table to store the string to id
    ///
    /// # Returns
    ///
    /// * `Result<(), redb::Error>` - The result of the commit
    pub fn commit_to(
        &self,
        txn: &redb::WriteTransaction,
        id_to_str_table_definition: TableDefinition<u64, String>,
        str_to_id_table_definition: TableDefinition<String, u64>,
    ) -> Result<(), redb::Error> {
        let mut id_to_str_table = txn.open_table(id_to_str_table_definition)?;
        let mut str_to_id_table = txn.open_table(str_to_id_table_definition)?;

        for entry in self.pending.iter() {
            let (s, id) = entry.pair();
            let id = *id as u64;
            let s = s.to_string();
            id_to_str_table.insert(id, s.clone())?;
            str_to_id_table.insert(s, id)?;

            // update pending to confirmed
            self.confirmed
                .insert(s.clone(), K::try_from_usize(id).unwrap());
            self.re_confirmed
                .insert(K::try_from_usize(id).unwrap(), s.clone());
        }

        self.re_pending.clear();
        self.pending.clear();

        Ok(())
    }

    pub fn from_database(
        txn: &redb::ReadTransaction,
        id_to_str_table_definition: TableDefinition<u64, String>,
        str_to_id_table_definition: TableDefinition<String, u64>,
    ) -> Result<Self, redb::Error> {
        let mut id_to_str_table = txn.open_table(id_to_str_table_definition)?;
        let mut str_to_id_table = txn.open_table(str_to_id_table_definition)?;

        let mut interner = Self {
            confirmed: DashMap::with_hasher(ahash::RandomState::default()),
            re_confirmed: DashMap::with_hasher(ahash::RandomState::default()),
            pending: DashMap::with_hasher(ahash::RandomState::default()),
            re_pending: DashMap::with_hasher(ahash::RandomState::default()),
            next_id: AtomicUsize::new(0),
        };
    }
}

use blake3::Hash;
use rkyv::{Archive, Deserialize, Serialize};
use std::{
    hash::Hasher,
    pin::Pin,
    sync::{Arc, OnceLock},
};
use tokio::io::AsyncRead;
use zako_digest::{Digest, blake3_hash::Blake3Hash};

use crate::{blob_range::BlobRange, cas_store::CasStore};

/// A runtime handle to a blob.
#[derive(Debug, Clone)]
pub struct BlobHandle {
    digest: Digest,
    state: BlobState,
}

impl PartialEq for BlobHandle {
    fn eq(&self, other: &Self) -> bool {
        self.digest == other.digest
    }
}

impl Eq for BlobHandle {}

impl std::hash::Hash for BlobHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.digest.blake3.as_bytes());
    }
}

#[derive(Debug, Clone)]
enum BlobState {
    Referenced,
    MemoryInlined { data: Arc<Vec<u8>> },
    // TODO: mmap Inlined
}

impl BlobHandle {
    pub fn new_referenced(digest: Digest) -> Self {
        Self {
            digest,
            state: BlobState::Referenced,
        }
    }

    pub fn new_memory_inlined(digest: Digest, data: Arc<Vec<u8>>) -> Self {
        Self {
            digest,
            state: BlobState::MemoryInlined { data },
        }
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn is_inlined(&self) -> bool {
        match self.state {
            BlobState::MemoryInlined { .. } => true,
            BlobState::Referenced => false,
        }
    }

    pub async fn open(
        &self,
        store: &CasStore,
        range: &BlobRange,
    ) -> eyre::Result<Pin<Box<dyn AsyncRead + Send>>> {
        Ok(match &self.state {
            // TODO: return inlined data
            BlobState::MemoryInlined { .. } => store.open(&self.digest, range).await?,
            // TODO: share the data
            BlobState::Referenced => store.open(&self.digest, range).await?,
        })
    }

    pub async fn read(&self, store: &CasStore, range: &BlobRange) -> eyre::Result<Vec<u8>> {
        Ok(match &self.state {
            BlobState::MemoryInlined { .. } => store.read(&self.digest, range).await?,
            // TODO: share the data
            BlobState::Referenced => store.read(&self.digest, range).await?,
        })
    }
}

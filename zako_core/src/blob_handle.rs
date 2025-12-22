use bitcode::{Decode, Encode};
use blake3::Hash;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use zako_digest::{Digest, blake3_hash::Blake3Hash};

use crate::cas_store::CasStore;

// TODO: Implement this for Deserialize
// Issue URL: https://github.com/moefra/zako/issues/14
#[derive(Debug, Clone, Hash, Copy, PartialEq, Eq, Deserialize, Serialize, Decode, Encode)]
pub struct BlobHandle {
    pub size: u64,

    pub hash: Hash,
}

impl BlobHandle {
    pub fn new(hash: u128, size: u64) -> Self {
        Self { hash, size }
    }
}

// 实现 Hash Trait，让它能作为 NodeKey 或 NodeValue 的一部分
impl Blake3Hash for BlobHandle {
    fn hash_into_blake3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        // Handle 的 Hash 就是内容的 Hash
        // 我们不需要再 Hash 一遍 size 或 inner 指针，只要内容 Hash 一样，它们就是同一个东西
        hasher.update(&self.hash.to_le_bytes());
    }
}

impl From<BlobHandle> for Digest {
    fn from(handle: BlobHandle) -> Self {
        Digest::new(handle.hash, handle.size)
    }
}

impl From<Digest> for BlobHandle {
    fn from(digest: Digest) -> Self {
        Self::new(digest.fast_xxhash3_128, digest.size_bytes)
    }
}

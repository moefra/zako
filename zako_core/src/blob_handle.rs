use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use zako_digest::{Digest, hash::XXHash3};

use crate::cas_store::CasStore;

// TODO: Implement this for Deserialize
// Issue URL: https://github.com/moefra/zako/issues/14
#[derive(Debug, Clone, Hash, Copy, PartialEq, Eq, Deserialize, Serialize, Decode, Encode)]
pub struct BlobHandle {
    /// 1. 身份 ID (XXH3)
    /// 冗余一份在外层，方便作为 Map Key，甚至不需要解引用 Arc
    pub hash: u128,

    /// 2. 元数据：大小
    /// 冗余在外层，方便快速做决策（比如：太大了就不读入内存）
    pub size: u64,
}

impl BlobHandle {
    pub fn new(hash: u128, size: u64) -> Self {
        Self { hash, size }
    }
}

// 实现 Hash Trait，让它能作为 NodeKey 或 NodeValue 的一部分
impl XXHash3 for BlobHandle {
    fn hash_into(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
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

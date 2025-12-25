use hone::node::Persistent;
use serde::{Deserialize, Serialize};
use zako_digest::blake3_hash::Blake3Hash;

use crate::{blob_handle::BlobHandle, context::BuildContext, path::interned::InternedNeutralPath};

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct File {
    pub path: InternedNeutralPath,
}

impl Blake3Hash for File {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(&self.path.interned().as_u64().to_le_bytes());
    }
}

#[derive(Debug, Clone, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct FileResult {
    /// 权限位 (对 TS 不重要，但对 shell 脚本重要)
    pub is_executable: bool,
    /// 是否是符号链接
    pub is_symlink: bool,
    /// 关键：CAS 句柄 (包含 Hash 和 数据指针)
    pub content: BlobHandle,
}

impl Blake3Hash for FileResult {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.is_executable.hash_into_blake3(hasher);
        self.content.hash_into_blake3(hasher);
    }
}

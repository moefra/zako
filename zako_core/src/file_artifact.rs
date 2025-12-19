use bitcode::{Decode, Encode};
use hone::node::Persistent;
use serde::{Deserialize, Serialize};
use zako_digest::hash::XXHash3;

use crate::{blob_handle::BlobHandle, context::BuildContext, path::interned::InternedNeutralPath};

#[derive(Debug, Clone)]
pub struct FileArtifact {
    /// 逻辑路径 "src/utils.ts"
    pub path: InternedNeutralPath,
    /// 权限位 (对 TS 不重要，但对 shell 脚本重要)
    pub is_executable: bool,
    /// 关键：CAS 句柄 (包含 Hash 和 数据指针)
    pub content: BlobHandle,
}

impl XXHash3 for FileArtifact {
    fn hash_into(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(&self.path.interned().as_u64().to_le_bytes());
        self.is_executable.hash_into(hasher);
        self.content.hash_into(hasher);
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Decode, Encode, PartialEq, Eq, Hash)]
pub struct RawFileArtifact {
    pub path: String,
    pub is_executable: bool,
    pub content: BlobHandle,
}

impl Persistent<BuildContext> for FileArtifact {
    type Persisted = RawFileArtifact;

    fn to_persisted(&self, ctx: &BuildContext) -> Option<Self::Persisted> {
        Some(RawFileArtifact {
            path: ctx.interner().resolve(self.path.interned()).to_string(),
            is_executable: self.is_executable,
            content: self.content.clone(),
        })
    }
    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Option<Self> {
        Some(FileArtifact {
            path: unsafe { InternedNeutralPath::from_raw(ctx.interner().get_or_intern(p.path)) },
            is_executable: p.is_executable,
            content: p.content,
        })
    }
}

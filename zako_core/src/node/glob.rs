use hone::node::Persistent;
use serde::{Deserialize, Serialize};
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    blob_handle::BlobHandle,
    context::BuildContext,
    intern::InternedAbsolutePath,
    path::interned::InternedNeutralPath,
    pattern::{InternedPattern, Pattern},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct Glob {
    pub base_path: InternedAbsolutePath,
    pub pattern: InternedPattern,
}

impl Blake3Hash for Glob {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(&self.base_path.interned.as_u64().to_le_bytes());
        self.pattern.hash_into_blake3(hasher);
    }
}

#[derive(Debug, Clone, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct GlobResult {
    pub paths: Vec<InternedNeutralPath>,
}

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

#[derive(Debug, Clone, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct GlobResult {
    pub paths: Vec<InternedNeutralPath>,
}

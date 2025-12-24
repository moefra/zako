use hone::node::Persistent;
use zako_digest::blake3_hash::Blake3Hash;

use crate::blob_handle::BlobHandle;

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct TranspileTs {
    /// For debug and error reporting.
    pub name: String,
    /// The code to transpile.
    pub code: BlobHandle,
}
impl Blake3Hash for TranspileTs {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.name.hash_into_blake3(hasher);
        self.code.hash_into_blake3(hasher);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct TranspileTsResult {
    pub code: String,
    pub source_map: Option<String>,
}

impl Blake3Hash for TranspileTsResult {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.code.hash_into_blake3(hasher);
        self.source_map.hash_into_blake3(hasher);
    }
}

use bitcode::{Decode, Encode};
use hone::node::Persistent;
use xxhash_rust::xxh3;
use zako_digest::hash::XXHash3;

use crate::{blob_handle::BlobHandle, context::BuildContext};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TranspileTs {
    /// For debug and error reporting.
    pub name: String,
    /// The code to transpile.
    pub code: BlobHandle,
}
impl XXHash3 for TranspileTs {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        self.name.hash_into(hasher);
        self.code.hash_into(hasher);
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode, Encode)]
pub struct RawTranspileTs {
    name: String,
    code: BlobHandle,
}
impl Persistent<BuildContext> for TranspileTs {
    type Persisted = RawTranspileTs;

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Option<Self> {
        Some(TranspileTs {
            name: p.name,
            code: p.code,
        })
    }
    fn to_persisted(&self, ctx: &BuildContext) -> Option<Self::Persisted> {
        Some(RawTranspileTs {
            name: self.name.clone(),
            code: self.code.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode, Encode)]
pub struct RawTranspileTsResult {
    pub code: String,
    pub source_map: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode, Encode)]
pub struct TranspileTsResult {
    pub code: String,
    pub source_map: Option<String>,
}

impl Persistent<BuildContext> for RawTranspileTsResult {
    type Persisted = RawTranspileTsResult;

    fn to_persisted(&self, ctx: &BuildContext) -> Option<Self::Persisted> {
        Some(RawTranspileTsResult {
            code: self.code.clone(),
            source_map: self.source_map.clone(),
        })
    }

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Option<Self> {
        Some(RawTranspileTsResult {
            code: p.code,
            source_map: p.source_map,
        })
    }
}

impl XXHash3 for TranspileTsResult {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        self.code.hash_into(hasher);
        self.source_map.hash_into(hasher);
    }
}

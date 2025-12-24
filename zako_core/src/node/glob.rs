use bitcode::{Decode, Encode};
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
    fn hash_into_blake3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(&self.base_path.as_u64().to_le_bytes());
        self.pattern.hash_into_blake3(hasher);
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Decode, Encode, PartialEq, Eq, Hash)]
pub struct RawGlob {
    pub base_path: String,
    pub pattern: Pattern,
}

impl Persistent<BuildContext> for Glob {
    type Persisted = RawGlob;

    fn to_persisted(&self, ctx: &BuildContext) -> Option<Self::Persisted> {
        Some(RawGlob {
            base_path: ctx
                .interner()
                .resolve(self.base_path.interned())
                .to_string(),
            pattern: self.pattern.resolve(ctx.interner()),
        })
    }

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Option<Self> {
        Some(Glob {
            base_path: unsafe {
                InternedAbsolutePath::from_interned_unchecked(
                    ctx.interner().get_or_intern(p.base_path),
                )
            },
            pattern: p.pattern.intern(ctx),
        })
    }
}

#[derive(Debug, Clone)]
pub struct GlobResult {
    pub paths: Vec<InternedNeutralPath>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Decode, Encode, PartialEq, Eq, Hash)]
pub struct RawGlobResult {
    pub paths: Vec<String>,
}

impl Persistent<BuildContext> for GlobResult {
    type Persisted = RawGlobResult;

    fn to_persisted(&self, ctx: &BuildContext) -> Option<Self::Persisted> {
        Some(RawGlobResult {
            paths: self
                .paths
                .iter()
                .map(|path| ctx.interner().resolve(path.interned()).to_string())
                .collect(),
        })
    }
    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Option<Self> {
        Some(GlobResult {
            paths: p
                .paths
                .iter()
                .map(|path| unsafe {
                    InternedNeutralPath::from_raw(ctx.interner().get_or_intern(path))
                })
                .collect(),
        })
    }
}

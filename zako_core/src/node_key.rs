use std::path::PathBuf;

use bitcode::{Decode, Encode};
use hone::node::{NodeKey, NodeValue, Persistent};
use serde::{Deserialize, Serialize};
use strum::{Display, IntoStaticStr};
use zako_digest::{Digest, hash::XXHash3};

use crate::{
    context::BuildContext,
    file_artifact::RawFileArtifact,
    id::InternedLabel,
    intern::InternedAbsolutePath,
    package::InternedArtifactId,
    path::interned::InternedNeutralPath,
    pattern::{InternedPattern, Pattern},
};

/// Zako 构建图的核心键
/// 要求: 极度紧凑 (Copy), 极速 Hash, 语义清晰
#[derive(Debug, Clone, PartialEq, Eq, Hash, IntoStaticStr)]
pub enum ZakoKey {
    /// use [::ignore] to glob files
    Glob {
        base_path: InternedAbsolutePath,
        pattern: InternedPattern,
    },
    /// Resolve a project file
    ResolveProject {
        path: PathBuf,
    },
    File {
        path: InternedNeutralPath,
    },
}

/// Raw zako key. Used for persistent.
#[derive(Debug, Clone, PartialEq, Eq, Hash, IntoStaticStr, Encode, Decode)]
pub enum RawZakoKey {
    Glob { base_path: String, pattern: Pattern },
    ResolveProject,
    File(String),
}

impl Persistent<BuildContext> for ZakoKey {
    type Persisted = RawZakoKey;

    fn to_persisted(&self, ctx: &BuildContext) -> Option<Self::Persisted> {
        Some(match self {
            ZakoKey::Glob { base_path, pattern } => Self::Persisted::Glob {
                base_path: ctx.interner().resolve(&base_path.interned()).to_string(),
                pattern: pattern.resolve(ctx.interner()),
            },
            ZakoKey::ResolveProject { path: _ } => return None,
            ZakoKey::File { path } => {
                Self::Persisted::File(ctx.interner().resolve(&path.interned()).to_string())
            }
        })
    }

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Option<Self> {
        Some(match p {
            RawZakoKey::Glob { base_path, pattern } => unsafe {
                ZakoKey::Glob {
                    base_path: InternedAbsolutePath::from_interned_unchecked(
                        ctx.interner().get_or_intern(base_path.as_str()),
                    ),
                    pattern: pattern.intern(ctx),
                }
            },
            RawZakoKey::ResolveProject {} => return None,
            RawZakoKey::File(path) => unsafe {
                ZakoKey::File {
                    path: InternedNeutralPath::from_raw(ctx.interner().get_or_intern(path)),
                }
            },
        })
    }
}

impl XXHash3 for ZakoKey {
    fn hash_into(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        let name: &'static str = self.into();
        hasher.update(name.as_bytes());

        match self {
            ZakoKey::Glob { base_path, pattern } => {
                hasher.update(&base_path.as_u64().to_le_bytes());
                pattern.hash_into(hasher);
            }
            ZakoKey::ResolveProject { path } => {
                hasher.update(path.to_string_lossy().to_string().as_bytes());
            }
            ZakoKey::File { path } => {
                hasher.update(&path.interned().as_u64().to_le_bytes());
            }
        }
    }
}

impl NodeKey<BuildContext> for ZakoKey {}

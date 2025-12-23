use std::path::PathBuf;

use bitcode::{Decode, Encode};
use hone::node::{NodeKey, NodeValue, Persistent};
use serde::{Deserialize, Serialize};
use strum::{Display, IntoStaticStr};
use zako_digest::{Digest, blake3_hash::Blake3Hash};

use crate::{
    context::BuildContext,
    id::InternedPath,
    intern::InternedAbsolutePath,
    node::{
        file::{File, RawFile, RawFileResult},
        glob::{Glob, RawGlob},
        resolve_project::{RawResolveProject, ResolveProject},
        transpile_ts::{RawTranspileTs, TranspileTs},
    },
    package::InternedArtifactId,
    path::interned::InternedNeutralPath,
    pattern::{InternedPattern, Pattern},
};

/// Zako 构建图的核心键
#[derive(Debug, Clone, PartialEq, Eq, Hash, IntoStaticStr)]
pub enum ZakoKey {
    /// use [::ignore] to glob files
    Glob(Glob),
    /// Resolve a project file
    ResolveProject(ResolveProject),
    File(File),
    TranspileTs(TranspileTs),
}

/// Raw zako key. Used for persistent.
#[derive(Debug, Clone, PartialEq, Eq, Hash, IntoStaticStr, Encode, Decode)]
pub enum RawZakoKey {
    Glob(RawGlob),
    ResolveProject(RawResolveProject),
    File(RawFile),
    TranspileTs(RawTranspileTs),
}

impl Persistent for ZakoKey {
    type Persisted = RawZakoKey;

    fn to_persisted(&self, ctx: &BuildContext) -> Option<Self::Persisted> {
        Some(match self {
            ZakoKey::Glob(key) => Self::Persisted::Glob(key.to_persisted(ctx)?),
            ZakoKey::ResolveProject(key) => Self::Persisted::ResolveProject(key.to_persisted(ctx)?),
            ZakoKey::File(file) => Self::Persisted::File(file.to_persisted(ctx)?),
            ZakoKey::TranspileTs(key) => Self::Persisted::TranspileTs(key.to_persisted(ctx)?),
        })
    }

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Option<Self> {
        Some(match p {
            RawZakoKey::Glob(raw) => ZakoKey::Glob(Glob::from_persisted(raw, ctx)?),
            RawZakoKey::ResolveProject(raw) => {
                ZakoKey::ResolveProject(ResolveProject::from_persisted(raw, ctx)?)
            }
            RawZakoKey::File(raw) => ZakoKey::File(File::from_persisted(raw, ctx)?),
            RawZakoKey::TranspileTs(raw) => {
                ZakoKey::TranspileTs(TranspileTs::from_persisted(raw, ctx)?)
            }
        })
    }
}

impl NodeKey<BuildContext> for ZakoKey {}

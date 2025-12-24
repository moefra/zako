use std::path::PathBuf;

use bitcode::{Decode, Encode};
use hone::node::{NodeKey, NodeValue, Persistent};
use rkyv::Archive;
use rkyv::{Deserialize, Serialize};
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
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    IntoStaticStr,
    rkyv::Deserialize,
    rkyv::Serialize,
    rkyv::Archive,
)]
pub enum ZakoKey {
    /// use [::ignore] to glob files
    Glob(Glob),
    /// Resolve a project file
    ResolveProject(ResolveProject),
    File(File),
    TranspileTs(TranspileTs),
}

impl NodeKey for ZakoKey {}

use std::path::PathBuf;

use hone::node::NodeKey;
use strum::IntoStaticStr;

use crate::node::{
    file::File, glob::Glob, parse_manifest::ParseManifest, resolve_project::ResolveProject,
    transpile_ts::TranspileTs,
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
    ParseManifest(ParseManifest),
}

impl NodeKey for ZakoKey {}

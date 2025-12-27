use derive_more::From;
use hone::node::NodeKey;
use strum::IntoStaticStr;

use crate::node::{
    file::File, glob::Glob, parse_manifest::ParseManifest, resolve_label::ResolveLabel,
    resolve_package::ResolvePackage, transpile_ts::TranspileTs,
};

/// The key of the building graph.
///
/// Keep the name rule, some other convenient ways are depend on them.
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
    From,
)]
pub enum ZakoKey {
    /// use [::ignore] to glob files
    Glob(Glob),
    /// Resolve a project file
    ResolvePackage(ResolvePackage),
    File(File),
    TranspileTs(TranspileTs),
    ParseManifest(ParseManifest),
    ResolveLabel(ResolveLabel),
}

impl NodeKey for ZakoKey {}

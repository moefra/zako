use crate::node::glob::GlobResult;
use crate::node::parse_manifest::ParseManifestResult;
use crate::node::transpile_ts::TranspileTsResult;
use crate::node::{file::FileResult, resolve_project::ResolveProjectResult};
use hone::node::NodeValue;
use strum::IntoStaticStr;

#[derive(Debug, Clone, IntoStaticStr, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub enum ZakoValue {
    Glob(GlobResult),
    ResolveProject(ResolveProjectResult),
    FileResult(FileResult),
    TranspileTsResult(TranspileTsResult),
    ParseManifestResult(ParseManifestResult),
}

impl NodeValue for ZakoValue {}

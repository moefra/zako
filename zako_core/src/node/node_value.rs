use crate::node::glob::GlobResult;
use crate::node::parse_manifest::ParseManifestResult;
use crate::node::transpile_ts::TranspileTsResult;
use crate::node::{
    file::FileResult, resolve_label::ResolveLabelResult, resolve_package::ResolvePackageResult,
};
use hone::node::NodeValue;
use strum::IntoStaticStr;

#[derive(Debug, Clone, IntoStaticStr, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub enum ZakoValue {
    Glob(GlobResult),
    ResolvePackage(ResolvePackageResult),
    FileResult(FileResult),
    TranspileTs(TranspileTsResult),
    ParseManifest(ParseManifestResult),
    ResolveLabel(ResolveLabelResult),
}

impl NodeValue for ZakoValue {}

use hone::node::Persistent;
use smol_str::SmolStr;
use std::path::PathBuf;
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    context::BuildContext,
    intern::InternedAbsolutePath,
    package::InternedPackageId,
    package_source::{PackageSource, ResolvedPackageSource},
    project::ResolvedProject,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveProject {
    pub package: InternedPackageId,
    pub source: PackageSource,
    pub root: Option<InternedAbsolutePath>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveProjectResult {
    pub project: ResolvedProject,
}

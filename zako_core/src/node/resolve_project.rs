use hone::node::Persistent;
use smol_str::SmolStr;
use std::path::PathBuf;
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    context::BuildContext, intern::InternedAbsolutePath, package::InternedPackageId,
    project::ResolvedProject,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveProject {
    pub package: InternedPackageId,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveProjectResult {
    pub root: SmolStr,
    pub project: ResolvedProject,
}

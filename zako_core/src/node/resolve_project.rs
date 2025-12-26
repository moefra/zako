
use crate::{
    intern::InternedAbsolutePath,
    package::InternedPackageId,
    package_source::PackageSource,
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

use crate::{
    intern::InternedAbsolutePath, package::ResolvedPackage, package_id::InternedPackageId,
    package_source::PackageSource,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolvePackage {
    pub package: InternedPackageId,
    pub source: PackageSource,
    pub root: Option<InternedAbsolutePath>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolvePackageResult {
    pub package: ResolvedPackage,
}

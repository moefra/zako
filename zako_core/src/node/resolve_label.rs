use ::std::hash::Hasher;

use crate::{
    id::Label, intern::InternedAbsolutePath, package::ResolvedPackage,
    package_id::InternedPackageId, package_source::PackageSource, target::Target,
};

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveLabel {
    pub package: ResolvedPackage,
    pub label: Label,
}

impl std::hash::Hash for ResolveLabel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.package.get_id().hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveLabelResult {
    pub target: Target,
}

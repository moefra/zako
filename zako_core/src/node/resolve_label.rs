use ::std::hash::Hasher;

use crate::{
    configured_project::ConfiguredPackage, id::Label,
    target::Target,
};

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveLabel {
    pub package: ConfiguredPackage,
    pub label: Label,
}

impl std::hash::Hash for ResolveLabel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.package.source.hash(state);
        self.package.package.get_id().hash(state);
        self.label.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveLabelResult {
    pub target: Target,
}

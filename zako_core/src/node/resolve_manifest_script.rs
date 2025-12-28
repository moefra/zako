use ::std::hash::Hasher;

use crate::{configured_project::ConfiguredPackage, id::Label, package::Package, target::Target};

#[derive(Debug, Clone, Hash, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveManifestScript {
    pub package: Package,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveManifestScriptResult {
    pub target: Package,
}

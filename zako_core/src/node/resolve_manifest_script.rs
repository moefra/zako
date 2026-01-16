use ::std::hash::Hasher;

use smol_str::SmolStr;

use crate::{configured_project::ConfiguredPackage, id::Label, package::Package, target::Target};

#[derive(Debug, Clone, PartialEq, Hash, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveManifestScript {
    pub package: Package,
    pub configure_script: Option<SmolStr>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveManifestScriptResult {
    pub target: Package,
}

use ::std::hash::Hasher;
use std::sync::Arc;

use smol_str::SmolStr;

use crate::{
    configured_project::ConfiguredPackage,
    id::Label,
    package::{Package, ResolvingPackage},
    target::Target,
};

#[derive(Debug, Clone, PartialEq, Hash, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveManifestScript {
    pub package: ResolvingPackage,
    pub configure_script: Option<SmolStr>,
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveManifestScriptResult {
    pub package: ResolvingPackage,
}

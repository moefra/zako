use ::std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use ::zako_digest::blake3_hash::Blake3Hash;

use crate::{
    context::{BuildContext, BuildContextError},
    global_state::GlobalState,
    intern::{InternedAbsolutePath, Interner},
    package::{Package, PackageResolveError, ResolvedPackage},
    package_source::{PackageSource, ResolvedPackageSource},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ConfiguredPackage {
    pub raw_package_blake3: ::zako_digest::blake3_hash::Hash,
    pub raw_source_blake3: ::zako_digest::blake3_hash::Hash,
    pub source_root_blake3: ::zako_digest::blake3_hash::Hash,
    pub source: ResolvedPackageSource,
    pub package: ResolvedPackage,
    pub source_root: InternedAbsolutePath,
}

impl Blake3Hash for ConfiguredPackage {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        (
            self.raw_package_blake3,
            self.raw_source_blake3,
            self.source_root_blake3,
        )
            .hash_into_blake3(hasher);
    }
}

impl ConfiguredPackage {
    pub fn get_context(
        &self,
        interner: &Interner,
        env: Arc<GlobalState>,
    ) -> Result<BuildContext, BuildContextError> {
        BuildContext::new_from_configured_project(self, interner, env)
    }
}

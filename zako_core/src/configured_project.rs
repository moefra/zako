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
    pub raw_source: PackageSource,
    pub source: ResolvedPackageSource,
    pub package: ResolvedPackage,
    pub source_root: InternedAbsolutePath,
}

impl ConfiguredPackage {
    pub fn to_blake3_compatible<'i>(
        &self,
        interner: &'i Interner,
    ) -> Result<(Package, PackageSource), PackageResolveError> {
        let package = self.package.to_raw(&interner)?;
        let source = self.raw_source.clone();
        Ok((package, source))
    }

    pub fn get_context(
        &self,
        interner: &Interner,
        env: Arc<GlobalState>,
    ) -> Result<BuildContext, BuildContextError> {
        BuildContext::new_from_configured_project(self, interner, env)
    }
}

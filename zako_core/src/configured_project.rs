use ::std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use ::zako_digest::blake3::Blake3Hash;

use crate::{
    context::{BuildContext, BuildContextError},
    global_state::GlobalState,
    intern::{InternedAbsolutePath, Interner},
    package::{Package, PackageResolveError, ResolvedPackage},
    package_source::{InternedPackageSource, PackageSource},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ConfiguredPackage {
    pub source: InternedPackageSource,
    pub package: ResolvedPackage,
}

use ahash::AHashMap;
use hone::FastMap;
use lasso::{Capacity, ThreadedRodeo};

use crate::{
    dependency::InternedPackageSource, id::InternedString, package::InternedPackage,
    project::InternedProject,
};

#[derive(Debug)]
pub struct BuildContext {
    interner: ThreadedRodeo<InternedString, ::ahash::RandomState>,
    packages: FastMap<InternedPackage, InternedProject>,
    package_source: FastMap<InternedPackage, InternedPackageSource>,
}

impl BuildContext {
    pub fn new() -> Self {
        Self {
            interner: ThreadedRodeo::with_capacity_and_hasher(
                Capacity::for_strings(1024),
                ::ahash::RandomState::new(),
            ),
            packages: FastMap::default(),
            package_source: FastMap::default(),
        }
    }

    pub fn interner(&self) -> &ThreadedRodeo<InternedString, ::ahash::RandomState> {
        &self.interner
    }

    pub fn interner_mut(&mut self) -> &mut ThreadedRodeo<InternedString, ::ahash::RandomState> {
        &mut self.interner
    }
}

pub type SharedBuildContext = std::sync::Arc<BuildContext>;

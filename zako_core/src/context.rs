use std::{ops::Deref, sync::Arc};

use ahash::AHashMap;
use hone::FastMap;
use lasso::{Capacity, ThreadedRodeo};
use thiserror::Error;

use crate::{
    dependency::InternedPackageSource, id::InternedString, package::InternedPackage,
    project::InternedProject,
};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BuildContextError {
    #[error("failed to call getrandom::u64 to get random context id")]
    FailedToGetRandomNumber(#[from] ::getrandom::Error),
}

#[derive(Debug)]
pub struct BuildContext {
    context_id: u64,
    interner: ThreadedRodeo<InternedString, ::ahash::RandomState>,
    packages: FastMap<InternedPackage, InternedProject>,
    package_source: FastMap<InternedPackage, InternedPackageSource>,
}

impl BuildContext {
    pub fn new() -> Result<Self, BuildContextError> {
        Ok(Self {
            context_id: getrandom::u64()?,
            interner: ThreadedRodeo::with_capacity_and_hasher(
                Capacity::for_strings(1024),
                ::ahash::RandomState::new(),
            ),
            packages: FastMap::default(),
            package_source: FastMap::default(),
        })
    }

    pub fn interner(&self) -> &ThreadedRodeo<InternedString, ::ahash::RandomState> {
        &self.interner
    }

    pub fn interner_mut(&self) -> &mut ThreadedRodeo<InternedString, ::ahash::RandomState> {
        unsafe { &mut *(&self.interner as *const _ as *mut _) }
    }

    pub fn context_id(&self) -> u64 {
        self.context_id
    }

    pub fn get_handle(self: Arc<Self>) -> ContextHandler {
        crate::context::ContextHandler::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct ContextHandler {
    context: Arc<BuildContext>,
}

impl ContextHandler {
    pub fn new(context: Arc<BuildContext>) -> Self {
        Self { context }
    }

    pub fn context(&self) -> &Arc<BuildContext> {
        &self.context
    }
}

impl Eq for BuildContext {}

impl PartialEq for BuildContext {
    fn eq(&self, other: &Self) -> bool {
        self.context_id() == other.context_id()
    }
}

impl Eq for ContextHandler {}

impl PartialEq for ContextHandler {
    fn eq(&self, other: &Self) -> bool {
        self.context.context_id() == other.context.context_id()
    }
}

impl Deref for ContextHandler {
    type Target = BuildContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

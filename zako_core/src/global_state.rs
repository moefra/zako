use std::{num::NonZeroUsize, sync::Arc};

use lasso::{Capacity, ThreadedRodeo};

use crate::{
    FastMap,
    context::BuildContext,
    intern::{InternedString, Interner},
    resource::ResourcePool,
};

#[derive(Debug)]
pub struct GlobalState {
    interner: Arc<ThreadedRodeo<InternedString, ::ahash::RandomState>>,
    resource_pool: Arc<ResourcePool>,
    /// The key is absolute path to the package root.
    packages: Arc<FastMap<InternedString, Arc<BuildContext>>>,
    // engine: Arc<hone::engine::Engine<>>,
}

impl GlobalState {
    pub fn new(resource_pool: ResourcePool) -> Self {
        Self {
            interner: Arc::new(ThreadedRodeo::with_capacity_and_hasher(
                Capacity::new(64, NonZeroUsize::new(1024).unwrap()), // 1k strings to start with
                ::ahash::RandomState::default(),
            )),
            resource_pool: Arc::new(resource_pool),
            packages: Arc::new(FastMap::default()),
            // engine: Arc::new(hone::engine::Engine::new()),
        }
    }

    pub fn interner<'c>(&'c self) -> &'c mut Interner {
        // it is thread safe to mutably borrow the interner here
        let interner = self.interner.as_ref();
        let interner = interner as &Interner;
        unsafe { &mut *(interner as *const _ as *mut _) }
    }

    pub fn resource_pool(&self) -> &ResourcePool {
        &self.resource_pool
    }

    pub fn packages(&self) -> &FastMap<InternedString, Arc<BuildContext>> {
        &self.packages
    }
}

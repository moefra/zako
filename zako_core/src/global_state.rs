use std::{num::NonZeroUsize, path::PathBuf, sync::Arc};

use lasso::{Capacity, ThreadedRodeo};
use sysinfo::System;
use tokio::runtime::{Builder, Runtime};

use crate::{
    FastMap,
    cas_store::CasStore,
    context::BuildContext,
    intern::{InternedString, Interner},
    local_cas::LocalCas,
    resource::{
        self, ResourcePool,
        heuristics::{determine_local_cas_path, determine_tokio_thread_stack_size},
    },
    worker::{
        oxc_worker::OxcTranspilerWorker,
        v8_worker::V8Worker,
        worker_pool::{PoolConfig, WorkerPool},
    },
};

#[derive(Debug, thiserror::Error)]
pub enum GlobalStateError {
    #[error("Get a worker pool error: {0}")]
    WorkerPoolError(#[from] crate::worker::worker_pool::WorkerPoolError),
    #[error("Get an io error: {0}")]
    IOError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct GlobalState {
    interner: Arc<ThreadedRodeo<InternedString, ::ahash::RandomState>>,
    resource_pool: Arc<ResourcePool>,
    /// The key is absolute path to the package root.
    packages: Arc<FastMap<InternedString, Arc<BuildContext>>>,
    tokio_runtime: Runtime,
    system: Arc<System>,
    cas_store: Arc<CasStore>,
    oxc_workers_pool: Arc<WorkerPool<OxcTranspilerWorker>>,
    v8_workers_pool: Arc<WorkerPool<V8Worker>>,
}

impl GlobalState {
    pub fn new(
        resource_pool: ResourcePool,
        oxc_workers_config: PoolConfig,
        v8_workers_config: PoolConfig,
    ) -> Result<Arc<Self>, GlobalStateError> {
        let cpu_count = resource_pool.get_cpu_count() as usize;
        let system = Arc::new(System::new_all());
        let this = Self {
            interner: Arc::new(ThreadedRodeo::with_capacity_and_hasher(
                Capacity::new(64, NonZeroUsize::new(1024).unwrap()), // 1k strings to start with
                ::ahash::RandomState::default(),
            )),
            resource_pool: Arc::new(resource_pool),
            packages: Arc::new(FastMap::default()),
            tokio_runtime: Builder::new_multi_thread()
                .worker_threads(cpu_count)
                .thread_name("zako-tokio-worker")
                .thread_stack_size(determine_tokio_thread_stack_size(&system))
                .build()?,
            system: system.clone(),
            cas_store: Arc::new(CasStore::new(
                Box::new(LocalCas::new(determine_local_cas_path(&system))),
                None,
                resource::heuristics::determine_memory_cache_size_for_cas(&system),
                resource::heuristics::determine_memory_ttl_for_cas(&system),
                resource::heuristics::determine_memory_tti_for_cas(&system),
            )),
            oxc_workers_pool: Arc::new(WorkerPool::new(oxc_workers_config)),
            v8_workers_pool: Arc::new(WorkerPool::new(v8_workers_config)),
        };

        let this = Arc::new(this);

        this.oxc_workers_pool.start(this.clone())?;
        this.v8_workers_pool.start(this.clone())?;

        Ok(this)
    }

    #[inline]
    pub fn interner<'c>(&'c self) -> &'c Interner {
        &self.interner
    }

    #[inline]
    pub fn resource_pool(&self) -> &ResourcePool {
        &self.resource_pool
    }

    #[inline]
    pub fn packages(&self) -> &FastMap<InternedString, Arc<BuildContext>> {
        &self.packages
    }

    #[inline]
    pub fn handle(&self) -> &tokio::runtime::Handle {
        self.tokio_runtime.handle()
    }

    #[inline]
    pub fn cas_store(&self) -> &CasStore {
        &self.cas_store
    }

    #[inline]
    pub fn oxc_workers_pool(&self) -> &WorkerPool<OxcTranspilerWorker> {
        &self.oxc_workers_pool
    }

    #[inline]
    pub fn v8_workers_pool(&self) -> &WorkerPool<V8Worker> {
        &self.v8_workers_pool
    }

    #[inline]
    pub fn system(&self) -> &System {
        &self.system
    }
}

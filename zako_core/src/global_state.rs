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
    resource::{self, ResourcePool},
    worker::{oxc_worker::OxcTranspilerWorker, v8_worker::V8Worker, worker_pool::WorkerPool},
};

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
    pub fn new(resource_pool: ResourcePool) -> Result<Self, std::io::Error> {
        let cpu_count = resource_pool.get_cpu_count() as usize;
        let system = Arc::new(System::new_all());
        Ok(Self {
            interner: Arc::new(ThreadedRodeo::with_capacity_and_hasher(
                Capacity::new(64, NonZeroUsize::new(1024).unwrap()), // 1k strings to start with
                ::ahash::RandomState::default(),
            )),
            resource_pool: Arc::new(resource_pool),
            packages: Arc::new(FastMap::default()),
            tokio_runtime: Builder::new_multi_thread()
                .worker_threads(cpu_count)
                .thread_name("zako-tokio-worker")
                .thread_stack_size(4 * 1024 * 1024)
                .build()?,
            system: system.clone(),
            cas_store: Arc::new(CasStore::new(
                Box::new(LocalCas::new(PathBuf::from("."))),
                None,
                resource::heuristics::determine_memory_cache_size_for_cas(&system),
                resource::heuristics::determine_memory_ttl_for_cas(&system),
                resource::heuristics::determine_memory_tti_for_cas(&system),
            )),
            oxc_workers_pool: Arc::new(WorkerPool::new(
                resource::heuristics::determine_oxc_workers_count(&system),
            )),
            v8_workers_pool: Arc::new(WorkerPool::new(
                resource::heuristics::determine_v8_workers_count(&system),
            )),
        })
    }

    #[inline]
    pub fn interner<'c>(&'c self) -> &'c Interner {
        &self.interner
    }

    pub fn resource_pool(&self) -> &ResourcePool {
        &self.resource_pool
    }

    pub fn packages(&self) -> &FastMap<InternedString, Arc<BuildContext>> {
        &self.packages
    }

    #[inline]
    pub fn handle(&self) -> &tokio::runtime::Handle {
        self.tokio_runtime.handle()
    }
}

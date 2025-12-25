use std::{num::NonZeroUsize, path::PathBuf, sync::Arc, sync::Weak};

use sysinfo::System;
use tokio::runtime::{Builder, Runtime};

use crate::{
    FastMap,
    cas_store::{CasStore, CasStoreOptions},
    context::BuildContext,
    intern::{InternedAbsolutePath, InternedString, Interner},
    local_cas::LocalCas,
    package::InternedPackageId,
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
    interner: Arc<crate::intern::Interner>,
    resource_pool: Arc<ResourcePool>,
    /// The key is absolute path to the package root.
    path_to_context: Arc<FastMap<InternedAbsolutePath, Weak<BuildContext>>>,
    package_id_to_path: Arc<FastMap<InternedPackageId, InternedAbsolutePath>>,
    tokio_runtime: Runtime,
    system: Arc<System>,
    cas_store: Arc<CasStore>,
    oxc_workers_pool: Arc<WorkerPool<OxcTranspilerWorker>>,
    v8_workers_pool: Arc<WorkerPool<V8Worker>>,
}

impl GlobalState {
    pub fn new(
        resource_pool: ResourcePool,
        cas_store_options: CasStoreOptions,
        oxc_workers_config: PoolConfig,
        v8_workers_config: PoolConfig,
    ) -> Result<Arc<Self>, GlobalStateError> {
        let cpu_count = resource_pool.get_cpu_count() as usize;
        let system = Arc::new(System::new_all());
        let this = Self {
            interner: Arc::new(Interner::new()),
            resource_pool: Arc::new(resource_pool),
            path_to_context: Arc::new(FastMap::default()),
            package_id_to_path: Arc::new(FastMap::default()),
            tokio_runtime: Builder::new_multi_thread()
                .worker_threads(cpu_count)
                .thread_name("zako-tokio-worker")
                .thread_stack_size(determine_tokio_thread_stack_size(&system))
                .build()?,
            system: system.clone(),
            cas_store: Arc::new(CasStore::new(
                Box::new(LocalCas::new(determine_local_cas_path(&system))),
                None,
                cas_store_options,
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
    pub fn path_to_context(&self) -> &FastMap<InternedAbsolutePath, Weak<BuildContext>> {
        &self.path_to_context
    }

    #[inline]
    pub fn package_id_to_path(&self) -> &FastMap<InternedPackageId, InternedAbsolutePath> {
        &self.package_id_to_path
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

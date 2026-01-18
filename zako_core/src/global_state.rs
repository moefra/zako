use std::{fmt::Debug, sync::Arc};

use sysinfo::System;
use tokio::runtime::{Builder, Runtime};
use tracing::info;

use crate::{
    ConcurrentMap,
    cas_store::{CasStore, CasStoreOptions},
    intern::{InternedAbsolutePath, InternedString, Interner},
    local_cas::LocalCas,
    package_id::InternedPackageId,
    resource::{
        ResourcePool,
        heuristics::{determine_local_cas_path, determine_tokio_thread_stack_size},
    },
    worker::{
        oxc_worker::OxcTranspilerWorker,
        v8worker::V8Worker,
        worker_pool::{PoolConfig, WorkerPool},
    },
};

#[derive(Debug, thiserror::Error)]
pub enum GlobalStateError {
    #[error("Get a worker pool error: {0}")]
    WorkerPoolError(#[from] crate::worker::worker_pool::WorkerPoolError),
    #[error("Get an io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Interner error: {0}")]
    InternerError(#[from] ::zako_interner::InternerError),
}

/// Some interned strings that are used everywhere.
#[derive(Debug)]
pub struct CommonInternedStrings {
    /// [crate::consts::DEFAULT_CONFIGURATION_MOUNT_POINT]
    pub config_mount: InternedString,
}

pub struct GlobalState {
    interner: Arc<crate::intern::Interner>,
    resource_pool: Arc<ResourcePool>,
    /// The key is absolute path to the package root.
    package_id_to_path: Arc<ConcurrentMap<InternedPackageId, InternedAbsolutePath>>,
    tokio_runtime: Runtime,
    system: Arc<System>,
    cas_store: Arc<CasStore>,
    oxc_workers_pool: Arc<WorkerPool<OxcTranspilerWorker>>,
    v8_workers_pool: Arc<WorkerPool<V8Worker>>,
    common_interneds: CommonInternedStrings,
}

impl Debug for GlobalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalState")
            .field("resource_pool", &self.resource_pool)
            .field("package_id_to_path", &self.package_id_to_path)
            .field("cas_store", &self.cas_store)
            .field("oxc_workers_pool", &self.oxc_workers_pool)
            .field("v8_workers_pool", &self.v8_workers_pool)
            .finish()
    }
}

impl GlobalState {
    #[must_use]
    pub fn new(
        system: System,
        resource_pool: ResourcePool,
        cas_store_options: CasStoreOptions,
        oxc_workers_config: PoolConfig,
        v8_workers_config: PoolConfig,
    ) -> Result<Arc<Self>, GlobalStateError> {
        let cpu_count = resource_pool.get_cpu_count() as usize;
        let system = Arc::new(system);
        let interner = Arc::new(Interner::new()?);

        let common_interneds = CommonInternedStrings {
            config_mount: interner
                .get_or_intern(crate::consts::DEFAULT_CONFIGURATION_MOUNT_POINT)?,
        };

        let this = Self {
            interner,
            resource_pool: Arc::new(resource_pool),
            package_id_to_path: Arc::new(ConcurrentMap::default()),
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
            common_interneds,
        };

        info!(
            "use local cas path {:?}",
            this.cas_store.get_local_cas().get_root()
        );

        let this = Arc::new(this);

        this.oxc_workers_pool.start(this.clone())?;
        this.v8_workers_pool.start(this.clone())?;

        Ok(this)
    }

    #[must_use]
    #[inline]
    pub fn interner<'c>(&'c self) -> &'c Interner {
        &self.interner
    }

    #[must_use]
    #[inline]
    pub fn resource_pool(&self) -> &ResourcePool {
        &self.resource_pool
    }

    #[must_use]
    #[inline]
    pub fn package_id_to_path(&self) -> &ConcurrentMap<InternedPackageId, InternedAbsolutePath> {
        &self.package_id_to_path
    }

    #[must_use]
    #[inline]
    pub fn handle(&self) -> &tokio::runtime::Handle {
        self.tokio_runtime.handle()
    }

    #[must_use]
    #[inline]
    pub fn cas_store(&self) -> &CasStore {
        &self.cas_store
    }

    #[must_use]
    #[inline]
    pub fn oxc_workers_pool(&self) -> &WorkerPool<OxcTranspilerWorker> {
        &self.oxc_workers_pool
    }

    #[must_use]
    #[inline]
    pub fn v8_workers_pool(&self) -> &WorkerPool<V8Worker> {
        &self.v8_workers_pool
    }

    #[must_use]
    #[inline]
    pub fn system(&self) -> &System {
        &self.system
    }

    #[must_use]
    #[inline]
    pub fn common_interneds(&self) -> &CommonInternedStrings {
        &self.common_interneds
    }
}

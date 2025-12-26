use std::{ops::Deref, sync::Arc};

use camino::Utf8PathBuf;
use sysinfo::System;
use thiserror::Error;
use tokio::runtime::Handle;

use crate::{
    cas_store::CasStore,
    global_state::GlobalState,
    intern::{InternedAbsolutePath, InternedString, Interner},
    package_source::{PackageSource, ResolvedPackageSource},
    worker::{oxc_worker::OxcTranspilerWorker, v8_worker::V8Worker, worker_pool::WorkerPool},
};

#[derive(Debug, Error)]
pub enum BuildContextError {
    #[error("the project root path `{0}` is not an absolute path")]
    ProjectRootNotAbsolute(Utf8PathBuf),
    #[error("failed to intern package source: {0}")]
    FailedToResolvePackageSource(String),
    #[error("Interner error: {0}")]
    InternerError(#[from] ::zako_interner::InternerError),
}

#[derive(Debug, Clone)]
pub struct BuildContext {
    project_root: InternedAbsolutePath,
    project_entry_name: InternedString,
    project_source: ResolvedPackageSource,
    env: Arc<GlobalState>,
}

impl BuildContext {
    /// Create a new BuildContext
    ///
    /// `project_source`: The package source of the project, it should be absolute path to the project,
    /// in the internal of zako it will be used as a unique id to identify the project.
    ///
    /// `project_root`: The root path of the project,it was usually built from the project_source
    ///
    /// `project_entry_name`: The entry point file name of the project,
    /// If it is None, use [crate::consts::PROJECT_MANIFEST_FILE_NAME] as entry point
    ///
    /// `env`: The global state
    pub fn new(
        project_root: &Utf8PathBuf,
        project_source: PackageSource,
        project_entry_name: Option<String>,
        env: Arc<GlobalState>,
    ) -> Result<Self, BuildContextError> {
        let interner = env.interner();

        let resolved = project_source
            .resolve(&project_root.as_path(), interner)
            .map_err(|err| BuildContextError::FailedToResolvePackageSource(err.to_string()))?;

        let entry = project_entry_name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(crate::consts::PROJECT_MANIFEST_FILE_NAME);

        Ok(Self {
            project_root: InternedAbsolutePath::from_interned(
                interner.get_or_intern(project_root.as_str())?,
                interner,
            )?
            .ok_or_else(|| BuildContextError::ProjectRootNotAbsolute(project_root.clone()))?,
            project_entry_name: interner.get_or_intern(entry)?,
            project_source: resolved,
            env,
        })
    }

    pub fn project_root(&self) -> InternedAbsolutePath {
        self.project_root
    }

    pub fn project_entry_name(&self) -> InternedString {
        self.project_entry_name
    }

    pub fn project_source(&self) -> &ResolvedPackageSource {
        &self.project_source
    }

    pub fn resource_pool(&self) -> &crate::resource::ResourcePool {
        self.env.resource_pool()
    }

    pub fn interner<'c>(&'c self) -> &'c Interner {
        self.env.interner()
    }

    pub fn get_handle(self: Arc<Self>) -> ContextHandler {
        crate::context::ContextHandler::new(self.clone())
    }

    pub fn handle(&self) -> &Handle {
        self.env.handle()
    }

    pub fn global_state(&self) -> Arc<GlobalState> {
        self.env.clone()
    }

    pub fn cas_store(&self) -> &CasStore {
        self.env.cas_store()
    }

    pub fn oxc_workers_pool(&self) -> &WorkerPool<OxcTranspilerWorker> {
        self.env.oxc_workers_pool()
    }

    pub fn v8_workers_pool(&self) -> &WorkerPool<V8Worker> {
        self.env.v8_workers_pool()
    }

    pub fn system(&self) -> &System {
        self.env.system()
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
        self.project_root() == other.project_root()
    }
}

impl Eq for ContextHandler {}

impl PartialEq for ContextHandler {
    fn eq(&self, other: &Self) -> bool {
        self.context.project_root() == other.context.project_root()
    }
}

impl Deref for ContextHandler {
    type Target = BuildContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

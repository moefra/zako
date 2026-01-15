use std::{ops::Deref, sync::Arc};

use ::eyre::ContextCompat;
use camino::Utf8PathBuf;
use sysinfo::System;
use thiserror::Error;
use tokio::runtime::Handle;

use crate::{
    cas_store::CasStore,
    configured_project::ConfiguredPackage,
    global_state::{CommonInternedStrings, GlobalState},
    intern::{Internable, InternedAbsolutePath, InternedString, Interner},
    package_source::{PackageSource, PackageSourceResolveArguments, ResolvedPackageSource},
    worker::{oxc_worker::OxcTranspilerWorker, v8worker::V8Worker, worker_pool::WorkerPool},
};

#[derive(Debug, Error)]
pub enum BuildContextError {
    #[error("the project root path `{0}` is not an absolute path")]
    ProjectRootNotAbsolute(Utf8PathBuf),
    #[error("failed to intern package source: {0}")]
    FailedToResolvePackageSource(String),
    #[error("Interner error: {0}")]
    InternerError(#[from] ::zako_interner::InternerError),
    #[error("other error: {0}")]
    Other(#[from] eyre::Report),
}

/// A context for building a package.
///
/// This is stateless, meaning it can built from information and it can copy easily.
#[derive(Debug, Clone)]
pub struct BuildContext {
    project_root: InternedAbsolutePath,
    project_entry_name: InternedString,
    project_source: ResolvedPackageSource,
    env: Arc<GlobalState>,
}

impl BuildContext {
    #[must_use]
    /// Create a new BuildContext
    ///
    /// `project_source`: The package source of the project, it should be absolute path to the project,
    /// in the internal of zako it will be used as a unique id to identify the project.
    ///
    /// `project_root`: The root path of the project,it was usually built from the project_source
    ///
    /// `project_entry_name`: The entry point file name of the project,
    /// If it is None, use [crate::consts::PACKAGE_MANIFEST_FILE_NAME] as entry point
    ///
    /// `env`: The global state
    pub fn new(
        project_root: &Utf8PathBuf,
        project_source: PackageSource,
        project_entry_name: Option<String>,
        env: Arc<GlobalState>,
    ) -> Result<Self, BuildContextError> {
        let interner = env.interner();

        let args = PackageSourceResolveArguments {
            interner,
            root_path: project_root,
        };

        let project_source = project_source
            .intern(&args)
            .map_err(|err| BuildContextError::FailedToResolvePackageSource(err.to_string()))?;

        let entry = project_entry_name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(crate::consts::PACKAGE_MANIFEST_FILE_NAME);

        Ok(Self {
            project_root: InternedAbsolutePath::from_interned(
                interner.get_or_intern(project_root.as_str())?,
                interner,
            )?
            .ok_or_else(|| BuildContextError::ProjectRootNotAbsolute(project_root.clone()))?,
            project_entry_name: interner.get_or_intern(entry)?,
            project_source,
            env,
        })
    }

    pub fn new_from_configured_project(
        configured_project: &ConfiguredPackage,
        interner: &Interner,
        env: Arc<GlobalState>,
    ) -> Result<Self, BuildContextError> {
        let project_root = configured_project.source_root;
        let project_source = configured_project.source.clone();
        let project_entry_name =
            interner.get_or_intern(crate::consts::PACKAGE_MANIFEST_FILE_NAME)?;
        Ok(Self {
            project_root,
            project_entry_name,
            project_source,
            env,
        })
    }

    #[inline]
    #[must_use]
    pub fn project_root(&self) -> InternedAbsolutePath {
        self.project_root
    }

    #[inline]
    #[must_use]
    pub fn project_entry_name(&self) -> InternedString {
        self.project_entry_name
    }

    #[inline]
    #[must_use]
    pub fn package_source(&self) -> &ResolvedPackageSource {
        &self.project_source
    }

    #[inline]
    #[must_use]
    pub fn resource_pool(&self) -> &crate::resource::ResourcePool {
        self.env.resource_pool()
    }

    #[inline]
    #[must_use]
    pub fn interner<'c>(&'c self) -> &'c Interner {
        self.env.interner()
    }

    #[inline]
    #[must_use]
    pub fn get_handle(self: Arc<Self>) -> ContextHandler {
        crate::context::ContextHandler::new(self.clone())
    }

    #[inline]
    #[must_use]
    pub fn handle(&self) -> &Handle {
        self.env.handle()
    }

    #[inline]
    #[must_use]
    pub fn global_state(&self) -> Arc<GlobalState> {
        self.env.clone()
    }

    #[inline]
    #[must_use]
    pub fn cas_store(&self) -> &CasStore {
        self.env.cas_store()
    }

    #[inline]
    #[must_use]
    pub fn oxc_workers_pool(&self) -> &WorkerPool<OxcTranspilerWorker> {
        self.env.oxc_workers_pool()
    }

    #[inline]
    #[must_use]
    pub fn v8_workers_pool(&self) -> &WorkerPool<V8Worker> {
        self.env.v8_workers_pool()
    }

    #[inline]
    #[must_use]
    pub fn system(&self) -> &System {
        self.env.system()
    }

    #[inline]
    #[must_use]
    pub fn common_interneds(&self) -> &CommonInternedStrings {
        self.env.common_interneds()
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

impl AsRef<Interner> for BuildContext {
    fn as_ref(&self) -> &Interner {
        self.interner()
    }
}

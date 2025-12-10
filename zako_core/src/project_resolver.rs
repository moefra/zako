use crate::engine::{Engine, EngineError};
use crate::id::{PackageId, PackageIdError};
use crate::path::NeutralPath;
use crate::project::{Project, ResolvedProject};
use crate::project_resolver::ProjectResolveError::{CircularDependency, FileNotExists, NotAFile};
use crate::sandbox::SandboxError;
use crate::v8error::V8Error;
use crate::zako_module_loader::{ModuleLoadError, ModuleSpecifier, ModuleType};
use ahash::AHashMap;
use std::cell::RefCell;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tracing::{event, instrument};

#[derive(Error, Debug)]
pub enum ProjectResolveError {
    #[error("the script file `{0}` does not exists")]
    FileNotExists(PathBuf),
    #[error("the path to project file `{0}` is not a file")]
    NotAFile(PathBuf),
    #[error("detect circular dependency when resolving project {0}")]
    CircularDependency(PathBuf),
    #[error("get an io error")]
    IOError(#[from] io::Error),
    #[error("get an sandbox error")]
    SandboxError(#[from] SandboxError),
    #[error("get an engine error")]
    EngineError(#[from] EngineError),
    #[error("get an module load error")]
    ModuleLoadError(#[from] ModuleLoadError),
    #[error("get an v8 engine error: {0:?}")]
    V8Error(V8Error),
    #[error("get an serde_v8 error")]
    V8SerdeError(#[from] deno_core::serde_v8::Error),
    #[error("get an error when try to prase the id of the package")]
    ParseError(#[from] PackageIdError),
    #[error("try to get parsed project `{0}` but not found in ProjectResolver.parsed")]
    NoExpectedProjectFound(PathBuf),
}

#[derive(Debug)]
pub struct ProjectResolver {
    engine: Engine,
    parsed: RefCell<AHashMap<PathBuf, Rc<Project>>>,
    resolved: RefCell<AHashMap<PathBuf, Rc<ResolvedProject>>>,
    resolving: RefCell<AHashMap<PathBuf, bool>>,
}

impl ProjectResolver {
    pub fn new(engine: Engine) -> Self {
        ProjectResolver {
            engine,
            parsed: RefCell::new(AHashMap::default()),
            resolved: RefCell::new(AHashMap::default()),
            resolving: RefCell::new(AHashMap::default()),
        }
    }

    #[instrument]
    fn resolve_project_inner(
        self: &mut Self,
        project_file_path: &NeutralPath,
    ) -> Result<Rc<Project>, ProjectResolveError> {
        let file = <NeutralPath as AsRef<Path>>::as_ref(project_file_path).canonicalize()?;

        if !file.exists() {
            return Err(FileNotExists(file));
        }

        if !file.is_file() {
            return Err(NotAFile(file));
        }

        if let Some(status) = self.resolving.borrow_mut().get(&file) {
            return if *status {
                Err(CircularDependency(file))
            } else {
                return Ok(self
                    .parsed
                    .borrow()
                    .get(&file)
                    .ok_or(ProjectResolveError::NoExpectedProjectFound(file.clone()))?)
                .cloned();
            };
        } else {
            self.resolving.borrow_mut().insert(file.clone(), true);
        }

        let project = self.engine.execute_module_and_then(
            &ModuleSpecifier::new_file_module(&file)?,
            |mut scope, _context, resolved_project| {
                let object = resolved_project.into();
                deno_core::serde_v8::from_v8::<Project>(&mut scope, object)
            },
        )??;

        event!(
            tracing::Level::INFO,
            "resolved project file `{}` successfully,get `{}`",
            file.display(),
            format!("{:?}", project)
        );

        self.resolving.borrow_mut().insert(file.clone(), false);

        Ok(Rc::new(project))
    }

    #[instrument]
    pub fn resolve_project(
        self: &mut Self,
        project_file_path: &NeutralPath,
    ) -> Result<(), ProjectResolveError> {
        let resolved = self.resolve_project_inner(project_file_path)?;

        let packageId = PackageId::from_str(&format!(
            "{}:{}@{}",
            resolved.group, resolved.artifact, resolved.version
        ))?;

        // process subpackages

        Ok(())
    }
}

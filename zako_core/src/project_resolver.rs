use crate::engine::Engine;
use crate::module_specifier::ModuleSpecifier;
use crate::project::ProjectExported;
use crate::project_resolver::ProjectResolveError::{
    CircularDependency, FileNotExists, IOError, NotAFile,
};
use ahash::AHashMap;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::{fs, io};
use thiserror::Error;
use tracing::instrument;

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
}

#[derive(Debug)]
pub struct ProjectResolver {
    engine: Engine,
    result: RefCell<AHashMap<PathBuf, ProjectExported>>,
    resolving: RefCell<AHashMap<PathBuf, bool>>,
}

impl ProjectResolver {
    pub fn new(engine: Engine) -> Self {
        ProjectResolver {
            engine,
            result: RefCell::new(AHashMap::default()),
            resolving: RefCell::new(AHashMap::default()),
        }
    }

    #[instrument]
    pub fn resolve_project(
        self: &Self,
        project_file_path: String,
    ) -> Result<(), ProjectResolveError> {
        let file = fs::canonicalize(project_file_path)?;

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
                Ok(())
            };
        } else {
            self.resolving.borrow_mut().insert(file.clone(), true);
        }

        self.engine
            .execute_module(&ModuleSpecifier::File(file.clone()));

        self.resolving.borrow_mut().insert(file, false);

        Ok(())
    }
}

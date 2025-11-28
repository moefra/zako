use crate::engine::Engine;
use crate::project::ProjectExported;
use crate::project_resolver::ProjectResolveError::{
    CircularDependency, FileNotExists, IOError, NotAFile,
};
use ahash::AHashMap;
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
    engine: Rc<Engine>,
    result: AHashMap<PathBuf, ProjectExported>,
    resolving: AHashMap<PathBuf, bool>,
}

impl ProjectResolver {
    pub fn new(engine: Rc<Engine>) -> Rc<Self> {
        Rc::from(ProjectResolver {
            engine,
            result: AHashMap::default(),
            resolving: AHashMap::default(),
        })
    }

    #[instrument]
    pub fn resolve_project(
        mut self: Rc<Self>,
        project_file_path: String,
    ) -> Result<(), ProjectResolveError> {
        let file = fs::canonicalize(project_file_path)?;

        if !file.exists() {
            return Err(FileNotExists(file));
        }

        if !file.is_file() {
            return Err(NotAFile(file));
        }

        if let Some(status) = self.resolving.get(&file) {
            return if *status {
                Err(CircularDependency(file))
            } else {
                Ok(())
            };
        } else {
            Rc::get_mut(&mut self)
                .expect("unsafe to get mut")
                .resolving
                .insert(file.clone(), true);
        }

        Ok(())
    }
}

use crate::engine::Engine;
use crate::project::ProjectExported;
use crate::project_resolver::ProjectResolveError::{
    CircularDependency, FileNotExists, IOError, NotAFile,
};
use ahash::AHashMap;
use std::fs;
use std::path::PathBuf;
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
    IOError(#[from] Box<dyn std::error::Error>),
}

#[derive(Debug)]
pub struct ProjectResolver {
    resolve_engine: Engine,
    resolved_result: AHashMap<PathBuf, ProjectExported>,
    resolving: AHashMap<PathBuf, bool>,
}

impl Default for ProjectResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectResolver {
    pub fn new() -> Self {
        todo!();
        /*
        ProjectResolver {
            resolved_result: AHashMap::default(),
            resolving: AHashMap::default(),
        }
         */
    }

    #[instrument]
    pub fn resolve_project(
        &mut self,
        project_file_path: String,
    ) -> Result<(), ProjectResolveError> {
        let file = fs::canonicalize(project_file_path).map_err(|x| IOError(x.into()))?;

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
            self.resolving.insert(file.clone(), true);
        }

        Ok(())
    }
}

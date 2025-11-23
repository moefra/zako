use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use crate::path::NeutralPath;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sandbox {
    root: PathBuf,
}

impl AsRef<OsStr> for Sandbox {
    fn as_ref(&self) -> &OsStr {
        self.root.as_ref()
    }
}


#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("try access file({target}) out of sandbox({sandbox})")]
    TryAccessFileOutOfSandbox{
        sandbox : PathBuf,
        target : PathBuf,
    },
    #[error("get an io error:{0}")]
    IoError(#[from] std::io::Error),
}

impl Sandbox {
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = path.as_ref();
        let root = std::fs::canonicalize(path)?;
        Ok(Self { root })
    }

    pub fn get_path_safe<R:AsRef<Path>,T:AsRef<NeutralPath>>(&self, referer: &R, target:&T) -> Result<PathBuf,SandboxError> {
        let referer = referer.as_ref();
        let target = target.as_ref();

        let referer = PathBuf::from(referer);
        let target = referer.join(target);
        let target = std::fs::canonicalize(target)?;

        if target.starts_with(&self.root) {
            Ok(target)
        } else {
            Err(SandboxError::TryAccessFileOutOfSandbox{
                sandbox:self.root.clone(),
                target
                }
            )
        }
    }

    pub fn join_path_for<P:AsRef<NeutralPath>>(&self, relative_path: &P) -> Result<PathBuf,SandboxError> {
        let relative_path = relative_path.as_ref();
        let combined_path = self.root.join(relative_path);
        let canonicalized_path = std::fs::canonicalize(&combined_path)?;

        if canonicalized_path.starts_with(&self.root) {
            Ok(canonicalized_path)
        } else {
            Err(SandboxError::TryAccessFileOutOfSandbox{
                sandbox:self.root.clone(),
                target: canonicalized_path}
            )
        }
    }
}

use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use crate::fs;
use crate::fs::VirtualFileError;

#[derive(Error, Debug)]
pub enum PathError {
    #[error("the input paths is empty")]
    EmptyString(),
    #[error("get error from virtual file operation: {0}")]
    VirtualFileError(#[from] VirtualFileError),
}

pub fn join(paths: &[&str]) -> Result<String, PathError> {
    if paths.len() == 0 {
        return Err(PathError::EmptyString());
    }

    let mut path = PathBuf::from(paths[0]);
    let paths = &paths[1..];

    for path_part in paths {
        path.push(path_part);
    }

    Ok(path.to_string_lossy().to_string())
}

pub fn filename(path: &str) -> Option<String> {
    if let Some(filename) = PathBuf::from(path).file_name() {
        Some(filename.to_string_lossy().to_string())
    } else {
        None
    }
}

pub fn parent(path: &str) -> Option<String> {
    if let Some(parent) = PathBuf::from(path).parent() {
        Some(parent.to_string_lossy().to_string())
    } else {
        None
    }
}

pub fn extname(path: &str) -> Option<String> {
    if let Some(extname) = PathBuf::from(path).extension() {
        Some(extname.to_string_lossy().to_string())
    } else {
        None
    }
}

pub fn normalize(path: &str) -> Result<String, PathError> {
    fs::normalize_path_to_string(Path::new(path)).map_err(|e|
        PathError::VirtualFileError(e))
}

pub fn is_link_out_of_dir(link_path_str: &str, target_str: &str) -> bool {
    fs::validate_symlink(link_path_str, target_str).is_err()
}

pub fn get_relative_path(from: &str, to: &str) -> Option<String> {
    pathdiff::diff_paths(to,from).map(|buf| buf.to_string_lossy().to_string())
}

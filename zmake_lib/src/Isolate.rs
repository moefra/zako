use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Isolate {
    root: PathBuf,
}

impl AsRef<Path> for Isolate {
    fn as_ref(&self) -> &Path {
        &self.root
    }
}

impl AsRef<PathBuf> for Isolate {
    fn as_ref(&self) -> &PathBuf {
        &self.root
    }
}

impl Isolate {
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = path.as_ref();
        let root = std::fs::canonicalize(path)?;
        Ok(Self { root })
    }

    /// 安全地检查路径是否在沙箱内，并返回相对路径
    pub fn get_existing_relative_path<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
        let target = path.as_ref();

        // 检查路径是否存在，确保其是文件并且获取绝对路径
        let target_abs = std::fs::canonicalize(target).ok()?;

        // 2. 剥离前缀
        match target_abs.strip_prefix(&self.root) {
            Ok(rel) => {
                for component in rel.components() {
                    if matches!(component, Component::ParentDir) {
                        return None; // 发现试图往上跳，拒绝
                    }
                }
                Some(rel.to_path_buf())
            }
            Err(_) => None,
        }
    }

    pub fn join<P: AsRef<Path>>(&self, relative_path: P) -> Option<PathBuf> {
        let path = relative_path.as_ref();

        // 1. 安全检查：相对路径不能包含绝对路径，也不能包含 ..
        if path.is_absolute() {
            return None;
        }

        for component in path.components() {
            match component {
                Component::Normal(_) => {}
                // 允许当前目录 .
                Component::CurDir => {}
                // 严禁向上跳级 ..
                Component::ParentDir => return None,
                // 严禁根目录 /
                Component::RootDir | Component::Prefix(_) => return None,
            }
        }

        // 2. 安全拼接
        Some(self.root.join(path))
    }
}

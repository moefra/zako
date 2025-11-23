use std::path::Component;
use std::path::{Path, PathBuf};
use thiserror::Error;
use url::Url;

use crate::digest::{Digest, DigestError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FsItem {
    File(Digest),
    Symlink(String),
    EmptyDirectory,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VirtualFsItem {
    relative_path: String,
    item: FsItem,
    is_executable: bool,
    is_readonly: bool,
}

#[derive(Error, Debug)]
pub enum VirtualFileError {
    #[error("the provided path try access parent path '..', which is not allowed")]
    TryAccessParentPath(),
    #[error("the provided path contains invalid UTF-8 characters")]
    InvalidUtf8Path(),
    #[error("the provided path is absolute, but a relative path is required")]
    PathIsAbsolute(),
    #[error("the provided path is empty")]
    EmptyPath(),
    #[error("symlink target is absolute, which is not allowed in sandbox")]
    SymlinkTargetAbsolute,
    #[error("symlink target try to access out of sandbox via '..'")]
    SymbolLinkEscape,
    #[error("Wrong digest: {0}")]
    WrongDigest(DigestError),
    #[error("the provided path is not normalized")]
    SourceFilePathNotNormalized(),
}

impl AsRef<String> for VirtualFsItem {
    fn as_ref(&self) -> &String {
        &self.relative_path
    }
}

impl VirtualFsItem {
    pub fn new(
        relative_path: PathBuf,
        item: FsItem,
        is_executable: bool,
        is_readonly: bool,
    ) -> Result<Self, VirtualFileError> {
        let normalized = normalize_path_to_string(&relative_path)?;

        if let FsItem::Symlink(ref target) = item {
            validate_symlink(&normalized, target)?;
        }

        Ok(Self {
            relative_path: normalized,
            item,
            is_executable,
            is_readonly,
        })
    }

    pub fn get_relative_path(&self) -> &String {
        &self.relative_path
    }

    pub fn get_digest(&self) -> &FsItem {
        &self.item
    }

    pub fn is_executable(&self) -> bool {
        self.is_executable
    }

    pub fn is_readonly(&self) -> bool {
        self.is_readonly
    }
}

/// 辅助函数：将 PathBuf 转换为标准化的 Unix 风格 String
pub fn normalize_path_to_string(path: &Path) -> Result<String, VirtualFileError> {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::Normal(c) => {
                let s = c.to_str().ok_or(VirtualFileError::InvalidUtf8Path())?;
                components.push(s);
            }
            std::path::Component::CurDir => continue,
            std::path::Component::ParentDir => {
                let _ = components
                    .pop()
                    .ok_or(VirtualFileError::TryAccessParentPath())?;
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(VirtualFileError::PathIsAbsolute());
            }
        }
    }

    if components.is_empty() {
        return Err(VirtualFileError::EmptyPath());
    }

    // 强制使用 '/' 连接
    Ok(components.join("/"))
}

/// 校验符号链接是否逃出沙箱
///
/// - `link_path_str`: 符号链接文件本身的相对路径 (e.g., "include/my_lib.h"). It should be normalized,meaning not "\.\."
///
/// - `target_str`: 符号链接指向的目标 (e.g., "../src/utils/internal.h")
pub fn validate_symlink(link_path_str: &str, target_str: &str) -> Result<(), VirtualFileError> {
    let target_path = Path::new(target_str);

    // 绝对路径严禁使用
    if target_path.is_absolute() {
        return Err(VirtualFileError::SymlinkTargetAbsolute);
    }

    let link_path = Path::new(link_path_str);

    // 计算初始深度
    let mut current_depth: i32 = 0;

    if let Some(parent) = link_path.parent() {
        for component in parent.components() {
            current_depth += match component {
                Component::Normal(_) => 1,
                _ => return Err(VirtualFileError::SourceFilePathNotNormalized()),
            }
        }
    }

    // 模拟路径游走
    for component in target_path.components() {
        current_depth += match component {
            Component::Normal(_) => 1,
            Component::CurDir => 0,
            Component::ParentDir => -1,
            Component::RootDir | Component::Prefix(_) => {
                return Err(VirtualFileError::SymlinkTargetAbsolute);
            }
        };

        if current_depth < 0 {
            return Err(VirtualFileError::SymbolLinkEscape);
        }
    }

    Ok(())
}

use crate::proto::fs::virtual_fs_item::Item as ProtoItem;
use std::convert::TryFrom;

impl TryFrom<crate::proto::fs::VirtualFsItem> for VirtualFsItem {
    type Error = VirtualFileError;

    fn try_from(proto: crate::proto::fs::VirtualFsItem) -> Result<Self, Self::Error> {
        let item = match proto.item {
            Some(ProtoItem::Digest(d)) => FsItem::File(
                d.try_into()
                    .map_err(|err| VirtualFileError::WrongDigest(err))?,
            ),
            Some(ProtoItem::SymlinkTarget(s)) => FsItem::Symlink(s),
            Some(ProtoItem::EmptyDirectory(_)) => FsItem::EmptyDirectory,
            None => return Err(VirtualFileError::EmptyPath()),
        };

        VirtualFsItem::new(
            PathBuf::from(proto.relative_path),
            item,
            proto.is_executable,
            proto.is_readonly,
        )
    }
}

use thiserror::Error;

use crate::digest::{Digest, DigestError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FsItem {
    File(Digest),
    Symlink(NeutralPath),
    EmptyDirectory,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VirtualFsItem {
    relative_path: NeutralPath,
    item: FsItem,
    is_executable: bool,
    is_readonly: bool,
}

#[derive(Error, Debug)]
pub enum VirtualFileError {
    #[error("the provided path or link target try access parent path '..', which is not allowed")]
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
    #[error("wrong digest: {0}")]
    WrongDigest(DigestError),
    #[error("the provided path is not normalized")]
    SourceFilePathNotNormalized(),
    #[error("get an path error")]
    PathError(#[from] PathError),
}

impl AsRef<NeutralPath> for VirtualFsItem {
    fn as_ref(&self) -> &NeutralPath {
        &self.relative_path
    }
}

impl VirtualFsItem {
    pub fn new(
        relative_path: NeutralPath,
        item: FsItem,
        is_executable: bool,
        is_readonly: bool,
    ) -> Result<Self, VirtualFileError> {
        if let FsItem::Symlink(ref target) = item {
            let target = relative_path.join(target)?;

            if target.is_in_dir(&relative_path) {
                return Err(VirtualFileError::TryAccessParentPath());
            }
        }

        Ok(Self {
            relative_path,
            item,
            is_executable,
            is_readonly,
        })
    }

    pub fn get_relative_path(&self) -> &NeutralPath {
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

use crate::path::{NeutralPath, PathError};
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
            Some(ProtoItem::SymlinkTarget(s)) => FsItem::Symlink(NeutralPath::new(s)?),
            Some(ProtoItem::EmptyDirectory(_)) => FsItem::EmptyDirectory,
            None => return Err(VirtualFileError::EmptyPath()),
        };

        VirtualFsItem::new(
            NeutralPath::new(proto.relative_path)?,
            item,
            proto.is_executable,
            proto.is_readonly,
        )
    }
}

use std::fmt::Display;

use crate::intern::Interner;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::id::{InternedAtom, is_xid_loose_ident};
use crate::intern::InternedString;

#[derive(thiserror::Error, Debug)]
pub enum PackageParseError {
    #[error("Invalid semver 2.0 format `{0}`:{1}")]
    VersionError(String, semver::Error),
    #[error("Invalid package format `{0}`:{1}")]
    InvalidFormat(String, String),
    #[error("Failed to parse part of package `{0}`:{1}")]
    IdParseError(String, #[source] crate::id::IdParseError),
}

/// 包版本号
///
/// 符合 SemVer 2.0.0 规范
///
/// 参考: https://semver.org/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedVersion(pub InternedString);

impl InternedVersion {
    pub fn try_parse(s: &str, interner: &Interner) -> Result<Self, PackageParseError> {
        _ = ::semver::Version::parse(s)
            .map_err(|err| PackageParseError::VersionError(s.to_string(), err))?;
        Ok(Self(interner.get_or_intern(s)))
    }
}

/// 包组 (Group ID)
///
/// 规则: 域名反写，由 '.' 分隔的 Atom
///
/// 例如: "moe.fra", "com.example"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedGroup(pub InternedString);

impl InternedGroup {
    pub fn try_parse(s: &str, interner: &Interner) -> Result<Self, PackageParseError> {
        if s.is_empty() {
            return Err(PackageParseError::InvalidFormat(
                s.to_string(),
                "Group cannot be empty".to_string(),
            ));
        }
        // 按点分割校验
        for part in s.split('.') {
            if !is_xid_loose_ident(part) {
                return Err(PackageParseError::InvalidFormat(
                    s.to_string(),
                    format!(
                        "the part `{}` does not pass the is_xid_loose_ident() check",
                        part
                    ),
                ));
            }
        }

        Ok(Self(interner.get_or_intern(s)))
    }
}

/// 完整的包标识符
///
/// 格式: `domain.reverse.group:name`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InternedArtifactId {
    pub group: InternedGroup,
    pub name: InternedAtom,
}

impl InternedArtifactId {
    /// 解析 "group:name" 格式字符串
    pub fn try_parse(s: &str, interner: &Interner) -> Result<Self, PackageParseError> {
        let (g_str, n_str) = s.split_once(':').ok_or_else(|| {
            PackageParseError::InvalidFormat(
                s.to_string(),
                "Invalid package id format. Expected 'group:name'".to_string(),
            )
        })?;

        Ok(Self {
            group: InternedGroup::try_parse(g_str, interner)?,
            name: InternedAtom::try_parse(n_str, interner)
                .map_err(|err| PackageParseError::IdParseError(s.to_string(), err))?,
        })
    }

    /// 还原为字符串 (Display)
    pub fn to_string(&self, interner: &Interner) -> String {
        format!(
            "{}:{}",
            interner.resolve(&self.group.0),
            interner.resolve(&self.name.0)
        )
    }
}

/// 一个包的完整信息
///
/// 由[InternedArtifactId]、[InternedVersion]组成
///
/// 长得像 `domain.reverse.group:name@version`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InternedPackage {
    pub name: InternedArtifactId,
    pub version: InternedVersion,
}

impl InternedPackage {
    pub fn new(name: InternedArtifactId, version: InternedVersion) -> Self {
        Self { name, version }
    }

    pub fn try_parse(s: &str, interner: &Interner) -> Result<Self, PackageParseError> {
        let (id_str, ver_str) = s.rsplit_once('@').ok_or_else(|| {
            PackageParseError::InvalidFormat(
                s.to_string(),
                "Invalid package format. Expected 'group:name@version'".to_string(),
            )
        })?;

        let name = InternedArtifactId::try_parse(id_str, interner)?;
        let version = InternedVersion::try_parse(ver_str, interner)?;

        Ok(Self { name, version })
    }
}

use crate::intern::Interner;

use zako_digest::blake3::Blake3Hash;

use crate::id::{InternedAtom, is_loose_ident};
use crate::intern::InternedString;

#[derive(thiserror::Error, Debug)]
pub enum PackageIdParseError {
    #[error("Invalid semver 2.0 format `{0}`:{1}")]
    VersionError(String, semver::Error),
    #[error("Invalid package format `{0}`:{1}")]
    InvalidFormat(String, String),
    #[error("Failed to parse part of package `{0}`:{1}")]
    IdParseError(String, #[source] crate::id::IdParseError),
    #[error("Interner error while parsing package: {0}")]
    InternerError(#[from] ::zako_interner::InternerError),
}

/// 包版本号
///
/// 符合 SemVer 2.0.0 规范
///
/// 参考: https://semver.org/
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive,
)]
pub struct InternedVersion(InternedString);

impl Blake3Hash for InternedVersion {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.0.hash_into_blake3(hasher);
    }
}

impl AsRef<InternedString> for InternedVersion {
    fn as_ref(&self) -> &InternedString {
        &self.0
    }
}

impl InternedVersion {
    pub fn try_parse(s: &str, interner: &Interner) -> Result<Self, PackageIdParseError> {
        _ = ::semver::Version::parse(s)
            .map_err(|err| PackageIdParseError::VersionError(s.to_string(), err))?;
        Ok(Self(interner.get_or_intern(s)?))
    }
}

/// 包组 (Group ID)
///
/// 规则: 域名反写，由 '.' 分隔的 Atom
///
/// 例如: "moe.fra", "com.example"
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive,
)]
pub struct InternedGroup(InternedString);

impl Blake3Hash for InternedGroup {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.0.hash_into_blake3(hasher);
    }
}

impl AsRef<InternedString> for InternedGroup {
    fn as_ref(&self) -> &InternedString {
        &self.0
    }
}

impl InternedGroup {
    pub fn try_parse(s: &str, interner: &Interner) -> Result<Self, PackageIdParseError> {
        if s.is_empty() {
            return Err(PackageIdParseError::InvalidFormat(
                s.to_string(),
                "Group cannot be empty".to_string(),
            ));
        }
        // 按点分割校验
        for part in s.split('.') {
            if !is_loose_ident(part) {
                return Err(PackageIdParseError::InvalidFormat(
                    s.to_string(),
                    format!(
                        "the part `{}` does not pass the is_xid_loose_ident() check",
                        part
                    ),
                ));
            }
        }

        Ok(Self(interner.get_or_intern(s)?))
    }
}

/// 完整的包标识符
///
/// 格式: `domain.reverse.group:name`
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive,
)]
pub struct InternedArtifactId {
    pub group: InternedGroup,
    pub name: InternedAtom,
}

impl Blake3Hash for InternedArtifactId {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.group.hash_into_blake3(hasher);
        self.name.hash_into_blake3(hasher);
    }
}

impl InternedArtifactId {
    /// 解析 "group:name" 格式字符串
    pub fn try_parse(s: &str, interner: &Interner) -> Result<Self, PackageIdParseError> {
        let (g_str, n_str) = s.split_once(':').ok_or_else(|| {
            PackageIdParseError::InvalidFormat(
                s.to_string(),
                "Invalid package id format. Expected 'group:name'".to_string(),
            )
        })?;

        Ok(Self {
            group: InternedGroup::try_parse(g_str, interner)?,
            name: InternedAtom::try_parse(n_str, interner)
                .map_err(|err| PackageIdParseError::IdParseError(s.to_string(), err))?,
        })
    }

    /// 还原为字符串 (Display)
    pub fn resolved(&self, interner: &Interner) -> Result<String, PackageIdParseError> {
        Ok(format!(
            "{}:{}",
            interner.resolve(&self.group)?,
            interner.resolve(&self.name)?
        ))
    }
}

/// 一个包的完整信息
///
/// 由[InternedArtifactId]、[InternedVersion]组成
///
/// 长得像 `domain.reverse.group:name@version`
#[derive(
    Debug, Clone, PartialEq, Copy, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive,
)]
pub struct InternedPackageId {
    pub name: InternedArtifactId,
    pub version: InternedVersion,
}

impl Blake3Hash for InternedPackageId {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.name.hash_into_blake3(hasher);
        self.version.hash_into_blake3(hasher);
    }
}

impl InternedPackageId {
    pub fn new(name: InternedArtifactId, version: InternedVersion) -> Self {
        Self { name, version }
    }

    pub fn try_parse(s: &str, interner: &Interner) -> Result<Self, PackageIdParseError> {
        let (id_str, ver_str) = s.rsplit_once('@').ok_or_else(|| {
            PackageIdParseError::InvalidFormat(
                s.to_string(),
                "Invalid package format. Expected 'group:name@version'".to_string(),
            )
        })?;

        let name = InternedArtifactId::try_parse(id_str, interner)?;
        let version = InternedVersion::try_parse(ver_str, interner)?;

        Ok(Self { name, version })
    }

    pub fn resolved(&self, interner: &Interner) -> Result<String, PackageIdParseError> {
        Ok(format!(
            "{}@{}",
            self.name.resolved(interner)?,
            interner.resolve(&self.version.0)?,
        ))
    }
}

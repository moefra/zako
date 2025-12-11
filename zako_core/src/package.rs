use crate::id::{Interner, is_xid_loose_ident};
use serde::{Deserialize, Serialize};

use crate::id::{InternedAtom, InternedString};

/// 包版本号
///
/// 符合 SemVer 2.0.0 规范
///
/// 参考: https://semver.org/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternedVersion(InternedString);

impl InternedVersion {
    pub fn try_parse(s: &str, interner: &mut Interner) -> Result<Self, String> {
        _ = ::semver::Version::parse(s).map_err(|err| format!("{:?}", err))?;
        Ok(Self(interner.get_or_intern(s)))
    }
}

/// 包组 (Group ID)
///
/// 规则: 域名反写，由 '.' 分隔的 Atom
///
/// 例如: "moe.fra", "com.example"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternedGroup(InternedString);

impl InternedGroup {
    pub fn try_parse(s: &str, interner: &mut Interner) -> Result<Self, String> {
        if s.is_empty() {
            return Err("Group cannot be empty".to_string());
        }
        // 按点分割校验
        for part in s.split('.') {
            if !is_xid_loose_ident(part) {
                return Err(format!("Invalid group format: '{}'", s));
            }
        }
        Ok(Self(interner.get_or_intern(s)))
    }
}

/// 包名 (Artifact Name)
///
/// 规则: 单个 Atom
///
/// 例如: "zako", "guava"
pub type InternedArtifactName = InternedAtom;

/// 完整的包标识符
///
/// 格式: `domain.reverse.group:name`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternedPackageId {
    pub group: InternedGroup,
    pub name: InternedArtifactName,
}

impl InternedPackageId {
    /// 解析 "group:name" 格式字符串
    pub fn try_parse(s: &str, interner: &mut Interner) -> Result<Self, String> {
        let (g_str, n_str) = s
            .split_once(':')
            .ok_or_else(|| "Invalid package format. Expected 'group:name'".to_string())?;

        Ok(Self {
            group: InternedGroup::try_parse(g_str, interner)?,
            name: InternedArtifactName::try_parse(n_str, interner)?,
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
/// 由[InternedPackageIdent]、[InternedVersion]组成
///
/// 长得像 `domain.reverse.group:name@version`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternedPackage {
    name: InternedPackageId,
    version: InternedVersion,
}

impl InternedPackage {
    pub fn new(name: InternedPackageId, version: InternedVersion) -> Self {
        Self { name, version }
    }
}

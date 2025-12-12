use serde::{Deserialize, Serialize};
use sha2::digest::typenum::int;

use crate::package::InternedPackage;

/// 判断字符串是否是合法的 XID 标识符
pub fn is_xid_ident(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut s = s.chars();

    if !unicode_ident::is_xid_start(s.next().unwrap()) {
        return false;
    }

    for c in s {
        if !unicode_ident::is_xid_continue(c) {
            return false;
        }
    }

    return true;
}

/// 判断字符串是否是宽松的 XID 标识符
///
/// 规则: 只能包含 XID 标识符,但是在首字符允许下划线 '_', 其他位置允许连字符 '-'
///
/// 这个规则一般是给文件路径开洞的
///
/// 更严格的规则请使用[is_xid_ident]函数
pub fn is_xid_loose_ident(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();

    // 检查首字符 (通常要求更严格)
    if let Some(first) = chars.next() {
        if !unicode_ident::is_xid_start(first) && first != '_' {
            return false;
        }
    }

    // 检查后续字符
    for c in chars {
        // 我们显式允许 '-'，因为文件名常用 (kebab-case)
        if !unicode_ident::is_xid_continue(c) && c != '-' {
            return false;
        }
    }

    return true;
}

/// 原子标识符。
///
/// 规则: 只能包含 XID 标识符,但是在首字符允许下划线 '_', 其他位置允许连字符 '-'
///
/// 例如: "main", "lib-utils", "_internal", "my-module"
///
/// 可通过[is_xid_loose_ident]函数校验合法性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternedAtom(pub InternedString);

impl InternedAtom {
    pub fn try_parse(s: &str, interner: &mut Interner) -> Result<Self, String> {
        if !is_xid_loose_ident(s) {
            return Err(format!("Invalid atom identifier: '{}'", s));
        }
        Ok(Self(interner.get_or_intern(s)))
    }

    pub fn as_str<'interner>(&self, interner: &'interner mut Interner) -> &'interner str {
        interner.resolve(&self.0)
    }
}

/// [InternedId]中的Path部分
///
/// 规则: 由斜杠 '/' 分隔的[InternedAtom]。允许为空，代表根路径
///
/// 例如: "src/ui/button", "core"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternedPath(InternedString);

impl InternedPath {
    pub fn try_parse(s: &str, interner: &mut Interner) -> Result<Self, String> {
        // 路径允许为空。字符串是合法的根包路径
        if s.is_empty() {
            return Ok(Self(interner.get_or_intern_static("")));
        }

        for segment in s.split('/') {
            if segment == "." || segment == ".." {
                return Err("Path cannot contain '.' or '..'".into());
            }
            // 校验每一段路径名必须合法
            if let Err(_) = InternedAtom::try_parse(segment, &mut *interner) {
                return Err(format!("Invalid path segment: '{}' in '{}'", segment, s));
            }
        }
        Ok(Self(interner.get_or_intern(s)))
    }
}

/// [InternedId]中的Target部分
///
/// 规则: 必须是[InternedAtom]
///
/// 例如: "main", "lib-utils", "test_suite"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternedTarget(InternedAtom);

impl InternedTarget {
    pub fn try_parse(s: &str, interner: &mut Interner) -> Result<Self, String> {
        let atom = InternedAtom::try_parse(s, interner)?;
        Ok(Self(atom))
    }
}

/// [InternedId]中的Package Reference部分
///
/// 规则: 不为空时，必须以@开头，其余部分必须是合法XID标识符；为空时，代表当前包。
///
/// 贮存的字符不包含@符号
///
/// 例如: "@zako","@curl","@openssl",""(当前包)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternedPackageRef(InternedString);

impl InternedPackageRef {
    pub fn try_parse(s: &str, interner: &mut Interner) -> Result<Self, String> {
        if s.is_empty() {
            // 允许空字符串，代表当前包
            return Ok(Self(interner.get_or_intern_static("")));
        }
        if !s.starts_with('@') || s.len() == 1 {
            return Err(
                "Package reference must start with '@' and not be empty, or be empty for current package".into(),
            );
        }
        // check
        let ident_str = &s[1..];
        if !is_xid_ident(ident_str) {
            return Err(format!(
                "Invalid package reference identifier: '{}'",
                ident_str
            ));
        }

        Ok(Self(interner.get_or_intern(ident_str)))
    }
}

/// 一个贮存的ID，包含包引用、路径和目标名称，例如`@curl//src:main`
///
/// 格式: `@<package_ref>//<path>/<subpath>/.../final_path:<target>`
///
/// 分别由[InternedPackageRef]、[InternedPath]和[InternedTarget]组成
///
/// NOTE AGAIN:其中@可为省略，代表当前包
///
/// NOTE AGAIN:其中//后面可以为空，代表包根路径
///
/// 最短的ID示例: `//:main` (当前包的根路径下的main目标)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InternedId {
    pub package_ref: InternedPackageRef,
    pub path: InternedPath,
    pub target: InternedTarget,
}

impl InternedId {
    pub fn new(
        package_ref: InternedPackageRef,
        path: InternedPath,
        target: InternedTarget,
    ) -> Self {
        Self {
            package_ref,
            path,
            target,
        }
    }

    pub fn try_parse(id: &str, interner: &mut Interner) -> Result<Self, String> {
        let parts: Vec<&str> = id.split(':').collect();
        if parts.len() != 2 {
            return Err("ID must contain exactly one ':' separating path and target".into());
        }

        let path_part = parts[0];
        let target_part = parts[1];

        let target = InternedTarget::try_parse(target_part, interner)?;

        let path_parts: Vec<&str> = path_part.split("//").collect();

        if path_parts.len() != 2 {
            return Err(
                "Path part must contain exactly one '//' separating package_ref and path".into(),
            );
        }

        let package_ref_str = path_parts[0];
        let path_str = path_parts[1];

        let package_ref = InternedPackageRef::try_parse(package_ref_str, interner)?;
        let path = InternedPath::try_parse(path_str, interner)?;

        Ok(Self {
            package_ref,
            path,
            target,
        })
    }
}

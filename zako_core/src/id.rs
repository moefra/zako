use ::zako_digest::blake3_hash::Blake3Hash;

use crate::intern::{InternedString, Interner};

/// Check a string match [Unicode Standard Annex #31](https://www.unicode.org/reports/tr31/)
///
/// Or more detailed,it reject empty string,and string with invalid xid start at first character or xid continue at following character.
///
/// This should be used all the way.
pub fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();

    if let Some(first) = chars.next() {
        if !unicode_ident::is_xid_start(first) && first != '_' {
            return false;
        }
    } else {
        return false;
    }

    for c in chars {
        if !unicode_ident::is_xid_continue(c) {
            return false;
        }
    }

    return true;
}

/// The is the loose version of function [is_xid_ident].
///
/// It reject empty string too,but allow `-` and `_` in any place of the input string.
///
/// This should be used only when the system contact with physics world,like name a ident from a real file name.
pub fn is_loose_ident(s: &str) -> bool {
    let mut chars = s.chars();

    if let Some(first) = chars.next() {
        if !unicode_ident::is_xid_start(first) && first != '_' && first != '-' {
            return false;
        }
    } else {
        return false;
    }

    for c in chars {
        if !unicode_ident::is_xid_continue(c) && c != '_' && c != '-' {
            return false;
        }
    }

    return true;
}

/// The is the more loose version of function [is_loose_ident].
///
/// It allows `.` in the input string.But does not allow the string that only have `.`.
///
/// It also disallow `.` followed by the string like `file.`.
pub fn is_more_loose_ident(s: &str) -> bool {
    let mut chars = s.chars();

    let mut last_char = if let Some(first) = chars.next() {
        if !unicode_ident::is_xid_start(first) && first != '_' && first != '-' && first != '.' {
            return false;
        }
        first
    } else {
        return false;
    };

    let mut only_dot = last_char == '.';

    for c in chars {
        if !unicode_ident::is_xid_continue(c) && c != '_' && c != '-' && c != '.' {
            return false;
        }
        last_char = c;
        only_dot = only_dot && last_char == '.';
    }

    return (!only_dot) && (last_char != '.');
}

#[derive(Debug, thiserror::Error)]
pub enum IdParseError {
    #[error("the id `{0}` (part `{1:?}`) not match loose XID ident rules")]
    NotMatchLooseXid(String, Option<String>),
    #[error("input is empty")]
    EmptyInput,
    #[error("the id `{0}` component `{1}` is invalid: {2}")]
    InvalidComponent(String, String, String),
    #[error("invalid id `{0}` format: {1}")]
    InvalidFormat(String, String),
    #[error("Interner error while parsing id: {0}")]
    InternerError(#[from] ::zako_interner::InternerError),
}

/// 原子标识符。
///
/// 规则: 只能包含 XID 标识符,但是在首字符允许下划线 '_', 其他位置允许连字符 '-'
///
/// 例如: "main", "lib-utils", "_internal", "my-module"
///
/// 可通过[is_xid_loose_ident]函数校验合法性
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive,
)]
pub struct InternedAtom(pub InternedString);

impl InternedAtom {
    pub fn try_parse(s: &str, interner: &Interner) -> Result<Self, IdParseError> {
        if !is_loose_ident(s) {
            return Err(IdParseError::NotMatchLooseXid(s.to_string(), None));
        }
        Ok(Self(interner.get_or_intern(s)?))
    }
}

impl Blake3Hash for InternedAtom {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.0.hash_into_blake3(hasher);
    }
}

/// [InternedId]中的Path(或者叫做Label)部分。允许为空。
///
/// 规则: 由斜杠 '/' 分隔的item组成，item需要通过[is_more_loose_ident]校验。
///
/// 尾随任何数量的斜杠都会被忽略。
///
/// 例如: "src/ui/button", "core"
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive,
)]
pub struct InternedPath(pub InternedString);

impl InternedPath {
    pub fn try_parse<'s>(
        s: &'s str,
        interner: &Interner,
    ) -> Result<(Self, Option<&'s str>), IdParseError> {
        // 路径允许为空。字符串是合法的根包路径
        if s.is_empty() {
            return Ok((Self(interner.get_or_intern("")?), None));
        }

        let mut last_segment = None;

        for segment in s.trim_end_matches('/').split('/') {
            if segment == "." || segment == ".." {
                return Err(IdParseError::InvalidComponent(
                    s.to_string(),
                    segment.to_string(),
                    "Label segments cannot be '.' or '..'".to_string(),
                ));
            }
            // 校验每一段路径名必须合法
            if !is_more_loose_ident(segment) {
                return Err(IdParseError::NotMatchLooseXid(
                    s.to_string(),
                    Some(segment.to_string()),
                ));
            }
            last_segment = Some(segment);
        }

        Ok((Self(interner.get_or_intern(s)?), last_segment))
    }
}

/// [InternedId]中的Target部分
///
/// 规则: 必须是[InternedAtom]
///
/// 例如: "main", "lib-utils", "test_suite"
#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct InternedTarget(pub InternedAtom);

impl InternedTarget {
    pub fn try_parse(s: &str, interner: &Interner) -> Result<Self, IdParseError> {
        let atom = InternedAtom::try_parse(s, interner)?;
        Ok(Self(atom))
    }
}

/// [InternedId]中的Package Reference部分
///
/// 规则: 不为空时，必须以@开头，其余部分必须是合法标识符(by [is_loose_ident])；为空时，代表当前包。
///
/// 贮存的字符不包含@符号
///
/// 例如: "@zako","@curl","@openssl",""(当前包)
#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct InternedPackageRef(pub InternedString);

impl InternedPackageRef {
    pub fn try_parse(s: &str, interner: &Interner) -> Result<Self, IdParseError> {
        if s.is_empty() {
            // 允许空字符串，代表当前包
            return Ok(Self(interner.get_or_intern("")?));
        }
        if !s.starts_with('@') || s.len() == 1 {
            return Err(IdParseError::InvalidFormat(
                s.to_string(),
                "Package reference must start with '@' and not be empty, or be empty for current package".to_string(),
            ));
        }
        // check
        let ident_str = &s[1..];
        if !is_loose_ident(ident_str) {
            return Err(IdParseError::NotMatchLooseXid(
                s.to_string(),
                Some(ident_str.to_string()),
            ));
        }

        Ok(Self(interner.get_or_intern(ident_str)?))
    }
}

/// 一个贮存的ID，包含包引用、路径和目标名称，例如`@curl//src:main`
///
/// 分别由[InternedPackageRef]、[InternedPath]和[InternedTarget]组成
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct Label {
    pub package_ref: InternedPackageRef,
    pub path: InternedPath,
    pub target: InternedTarget,
}

impl Label {
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

    pub fn resolved(&self, interner: &Interner) -> Result<String, IdParseError> {
        Ok(format!(
            "{}//{}:{}",
            interner.resolve(&self.package_ref.0)?,
            interner.resolve(&self.path.0)?,
            interner.resolve(&self.target.0.0)?
        ))
    }

    /// 格式: `@<package_ref>//<path>/<subpath>/.../final_path:<target>`
    ///
    /// 输出始终是明确的，解析的，即不需要再补充输出。所有默认值已揭晓。
    ///
    /// NOTE:其中@可为省略，代表当前包
    ///
    /// NOTE again:其中//后面可以为空，代表包根路径
    ///
    /// NOTE again:其中:后面可以为空，默认为最后一个label的名字
    ///
    /// Examples:
    /// - `//:main`: 当前包的根路径下的main目标
    /// - `//src` 代表src路径下的`src`目标，即等价于`//src:src`
    /// - `@curl//:main`: curl包的根路径下的main目标
    /// - `@curl//src:lib`: curl包的src路径下的lib目标
    /// - `@curl//crypto`: curl包的crypto路径下的crypto目标
    /// - `@curl//crypto/src:core`: curl包的crypto路径下的src路径下的core目标
    /// - `@curl//crypto/src:src`: curl包的crypto路径下的src路径下的src目标
    pub fn try_parse(id: &str, interner: &Interner) -> Result<Self, IdParseError> {
        let (package_ref, path) = id.split_once("//").ok_or_else(|| {
            IdParseError::InvalidFormat(
                id.to_string(),
                "ID must contain exactly one '//' separating package_ref and path".to_string(),
            )
        })?;
        let package_ref = InternedPackageRef::try_parse(package_ref, interner)?;

        let (path, target) = path.split_once(':').unwrap_or((path, ""));

        let (path, last_segment) = InternedPath::try_parse(path, interner)?;

        let target = if target.is_empty() {
            match last_segment {
                Some(segment) => InternedTarget::try_parse(segment, interner)?,
                None => {
                    return Err(IdParseError::InvalidFormat(
                        id.to_string(),
                        "if no target provided, the label must not be empty".to_string(),
                    ));
                }
            }
        } else {
            InternedTarget::try_parse(target, interner)?
        };

        Ok(Self {
            package_ref,
            path,
            target,
        })
    }
}

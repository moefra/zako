use std::{fmt::Display, str::FromStr};

use semver::{Comparator, Error, Op, VersionReq};
use smol_str::SmolStr;
use unicode_ident::is_xid_start;

/// This function checks if a string is a valid `name`.
pub fn check_part(part: &str) -> bool {
    let mut chars = part.chars();

    if let Some(start) = chars.next()
        && is_xid_start(start)
    {
        while let Some(char) = chars.next() {
            if !(is_xid_start(char) || char == '-') {
                return false;
            }
        }
        true
    } else {
        false
    }
}

/// Groups is combined by at least one `name`.
///
/// It looks like `com.example`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Group {
    pub parts: Vec<SmolStr>,
}

/// `name` is combined by group and a name.
///
/// It looks like `com.example:name`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub group: Group,
    pub name: SmolStr,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// `arch:`
    ///
    /// CPU architecture
    ///
    /// e.g. `x64`
    Architecture,
    /// `os:`
    ///
    /// Operation system
    ///
    /// e.g. `linux`
    OperatingSystem,
    /// `tool:`
    ///
    /// Tool
    ///
    /// e.g. `gcc`
    Tool,
    /// `pkg:`
    ///
    /// Package
    ///
    /// e.g. `curl`
    Package,
    /// `res:`
    ///
    /// Any resource, text or binary
    ///
    /// e.g. `icudata` or `source_code/*.rs`
    Resource,
    /// `cfg:`
    ///
    /// Configuration for building,like `debug` or `release_with_dbginfo`
    Config,
    /// `feat:`
    ///
    /// Feature for anything(usually for tool and package),like `cxx20` or `ssl_support` or `msvc-abi`
    Feature,
    /// `rule:`
    ///
    /// Rule for building,like `cc_binary`
    Rule,
}

/// `UniqueIdReq` is combined by group and a name and a version req.
///
/// It looks like `com.example:name@^1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniqueIdReq {
    pub ty: Type,
    pub name: Name,
    pub version: semver::VersionReq,
}

impl From<UniqueId> for UniqueIdReq {
    fn from(value: UniqueId) -> Self {
        let cmp = Comparator {
            op: Op::Exact,
            major: value.version.major,
            minor: Some(value.version.minor),
            patch: Some(value.version.patch),
            pre: value.version.pre,
        };

        UniqueIdReq {
            ty: value.ty,
            name: value.name,
            version: VersionReq {
                comparators: vec![cmp],
            },
        }
    }
}

/// `UniqueId` is combined by group and a name and a version.
///
/// It looks like `com.example:name@1.0.0`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UniqueId {
    pub ty: Type,
    pub name: Name,
    pub version: semver::Version,
}

/// AnyId is a id can have range or unique version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnyId {
    Range(UniqueIdReq),
    Unique(UniqueId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MismatchIdTypeError();

impl Display for MismatchIdTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expect an AnyId::Unique but get an AnyId::Range when try_into() called"
        )
    }
}

impl ::std::error::Error for MismatchIdTypeError {}

impl AnyId {
    pub fn get_type(&self) -> Type {
        match self {
            AnyId::Range(id) => id.ty.clone(),
            AnyId::Unique(id) => id.ty.clone(),
        }
    }

    pub fn get_name(&self) -> Name {
        match self {
            AnyId::Range(id) => id.name.clone(),
            AnyId::Unique(id) => id.name.clone(),
        }
    }

    pub fn get_version_req(&self) -> semver::VersionReq {
        match self {
            AnyId::Range(id) => id.version.clone(),
            AnyId::Unique(id) => VersionReq {
                comparators: vec![Comparator {
                    op: Op::Exact,
                    major: id.version.major,
                    minor: Some(id.version.minor),
                    patch: Some(id.version.patch),
                    pre: id.version.pre.clone(),
                }],
            },
        }
    }

    pub fn get_version(&self) -> Option<semver::Version> {
        match self {
            AnyId::Range(_) => None,
            AnyId::Unique(id) => Some(id.version.clone()),
        }
    }
}

impl From<UniqueId> for AnyId {
    fn from(id: UniqueId) -> Self {
        AnyId::Unique(id)
    }
}

impl From<UniqueIdReq> for AnyId {
    fn from(id: UniqueIdReq) -> Self {
        AnyId::Range(id)
    }
}

impl TryFrom<AnyId> for UniqueId {
    type Error = MismatchIdTypeError;

    fn try_from(value: AnyId) -> Result<Self, Self::Error> {
        match value {
            AnyId::Unique(id) => Ok(id),
            AnyId::Range(_) => Err(MismatchIdTypeError()),
        }
    }
}

impl From<AnyId> for UniqueIdReq {
    fn from(value: AnyId) -> Self {
        match value {
            AnyId::Range(range) => range,
            AnyId::Unique(id) => id.into(),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseIdError {
    #[error("empty input")]
    EmptyInput,
    #[error("missing type separator ':'")]
    MissingTypeSeparator,
    #[error("invalid type prefix: {0}")]
    InvalidType(String),
    #[error("missing name separator ':'")]
    MissingNameSeparator,
    #[error("invalid group part: {0}")]
    InvalidGroupPart(String),
    #[error("empty group")]
    EmptyGroup,
    #[error("invalid name: {0}")]
    InvalidName(String),
    #[error("invalid version: {0}")]
    InvalidVersion(String),
}

impl FromStr for Type {
    type Err = ParseIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "arch" => Ok(Type::Architecture),
            "os" => Ok(Type::OperatingSystem),
            "tool" => Ok(Type::Tool),
            "pkg" => Ok(Type::Package),
            "res" => Ok(Type::Resource),
            "cfg" => Ok(Type::Config),
            "feat" => Ok(Type::Feature),
            "rule" => Ok(Type::Rule),
            _ => Err(ParseIdError::InvalidType(s.to_string())),
        }
    }
}

impl FromStr for AnyId {
    type Err = ParseIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseIdError::EmptyInput);
        }

        // Parse type prefix (e.g., "pkg:")
        let (ty_str, rest) = s
            .split_once(':')
            .ok_or(ParseIdError::MissingTypeSeparator)?;
        let ty: Type = ty_str.parse()?;

        // Parse version (e.g., "@1.0.0" or "@^1.0")
        // treat no version as "@*"
        let (name_part, version_str) = rest.rsplit_once('@').unwrap_or((rest, "*"));

        // Parse group and name (e.g., "com.example:name")
        let (group_str, name_str) = name_part
            .rsplit_once(':')
            .ok_or(ParseIdError::MissingNameSeparator)?;

        // Parse group parts
        let group_parts: Vec<SmolStr> = group_str
            .split('.')
            .map(|part| {
                if check_part(part) {
                    Ok(SmolStr::new(part))
                } else {
                    Err(ParseIdError::InvalidGroupPart(part.to_string()))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        if group_parts.is_empty() {
            return Err(ParseIdError::EmptyGroup);
        }

        // Validate name
        if !check_part(name_str) {
            return Err(ParseIdError::InvalidName(name_str.to_string()));
        }

        let name = Name {
            group: Group { parts: group_parts },
            name: SmolStr::new(name_str),
        };

        // Try to parse as exact version first, then as version req
        if let Ok(version) = semver::Version::parse(version_str) {
            Ok(AnyId::Unique(UniqueId { ty, name, version }))
        } else if let Ok(version) = semver::VersionReq::parse(version_str) {
            Ok(AnyId::Range(UniqueIdReq { ty, name, version }))
        } else {
            Err(ParseIdError::InvalidVersion(version_str.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unique_id() {
        let id: AnyId = "pkg:com.example:mypackage@1.0.0".parse().unwrap();
        assert!(matches!(id, AnyId::Unique(_)));
        assert_eq!(id.get_type(), Type::Package);
        assert_eq!(id.get_name().name.as_str(), "mypackage");
        assert_eq!(id.get_name().group.parts.len(), 2);
        assert_eq!(id.get_name().group.parts[0].as_str(), "com");
        assert_eq!(id.get_name().group.parts[1].as_str(), "example");
        assert_eq!(id.get_version(), Some(semver::Version::new(1, 0, 0)));
    }

    #[test]
    fn test_parse_range_id() {
        let id: AnyId = "pkg:com.example:mypackage@^1.0".parse().unwrap();
        assert!(matches!(id, AnyId::Range(_)));
        assert_eq!(id.get_type(), Type::Package);
        assert_eq!(id.get_name().name.as_str(), "mypackage");
        assert_eq!(id.get_version(), None);
    }

    #[test]
    fn test_parse_no_version_id() {
        let id: AnyId = "pkg:com.example:mypackage".parse().unwrap();
        assert!(matches!(id, AnyId::Range(_)));
        assert_eq!(id.get_type(), Type::Package);
        assert_eq!(id.get_name().name.as_str(), "mypackage");
        assert_eq!(id.get_version(), None);
    }

    #[test]
    fn test_parse_all_types() {
        assert_eq!(
            "arch:cpu:amd-x@1.0.0".parse::<AnyId>().unwrap().get_type(),
            Type::Architecture
        );
        assert_eq!(
            "os:linux:ubuntu@1.0.0".parse::<AnyId>().unwrap().get_type(),
            Type::OperatingSystem
        );
        assert_eq!(
            "tool:build:gcc@1.0.0".parse::<AnyId>().unwrap().get_type(),
            Type::Tool
        );
        assert_eq!(
            "pkg:com:curl@1.0.0".parse::<AnyId>().unwrap().get_type(),
            Type::Package
        );
        assert_eq!(
            "res:data:icudata@1.0.0"
                .parse::<AnyId>()
                .unwrap()
                .get_type(),
            Type::Resource
        );
        assert_eq!(
            "cfg:build:debug@1.0.0".parse::<AnyId>().unwrap().get_type(),
            Type::Config
        );
        assert_eq!(
            "feat:cxx:support-ssl@1.0.0"
                .parse::<AnyId>()
                .unwrap()
                .get_type(),
            Type::Feature
        );
        assert_eq!(
            "rule:build:cc-binary@1.0.0"
                .parse::<AnyId>()
                .unwrap()
                .get_type(),
            Type::Rule
        );
    }

    #[test]
    fn test_parse_error_empty_input() {
        let result: Result<AnyId, _> = "".parse();
        assert!(matches!(result, Err(ParseIdError::EmptyInput)));
    }

    #[test]
    fn test_parse_error_missing_type_separator() {
        let result: Result<AnyId, _> = "pkgcom.example:name@1.0.0".parse();
        assert!(matches!(result, Err(ParseIdError::InvalidType(_))));
    }

    #[test]
    fn test_parse_error_invalid_type() {
        let result: Result<AnyId, _> = "invalid:com.example:name@1.0.0".parse();
        assert!(matches!(result, Err(ParseIdError::InvalidType(_))));
    }

    #[test]
    fn test_parse_error_missing_name_separator() {
        let result: Result<AnyId, _> = "pkg:com.example@1.0.0".parse();
        assert!(matches!(result, Err(ParseIdError::MissingNameSeparator)));
    }

    #[test]
    fn test_parse_error_invalid_group_part() {
        let result: Result<AnyId, _> = "pkg:123.example:name@1.0.0".parse();
        assert!(matches!(result, Err(ParseIdError::InvalidGroupPart(_))));
    }

    #[test]
    fn test_parse_error_invalid_name() {
        let result: Result<AnyId, _> = "pkg:com.example:123name@1.0.0".parse();
        assert!(matches!(result, Err(ParseIdError::InvalidName(_))));
    }

    #[test]
    fn test_parse_error_invalid_version() {
        let result: Result<AnyId, _> = "pkg:com.example:name@invalid".parse();
        assert!(matches!(result, Err(ParseIdError::InvalidVersion(_))));
    }

    #[test]
    fn test_parse_with_hyphen_in_name() {
        let id: AnyId = "pkg:com.example:my-package@1.0.0".parse().unwrap();
        assert_eq!(id.get_name().name.as_str(), "my-package");
    }

    #[test]
    fn test_parse_single_group_part() {
        let id: AnyId = "pkg:example:name@1.0.0".parse().unwrap();
        assert_eq!(id.get_name().group.parts.len(), 1);
        assert_eq!(id.get_name().group.parts[0].as_str(), "example");
    }

    #[test]
    fn test_parse_version_req_variants() {
        assert!(matches!(
            "pkg:com:name@>=1.0.0".parse::<AnyId>().unwrap(),
            AnyId::Range(_)
        ));
        assert!(matches!(
            "pkg:com:name@~1.0.0".parse::<AnyId>().unwrap(),
            AnyId::Range(_)
        ));
        assert!(matches!(
            "pkg:com:name@*".parse::<AnyId>().unwrap(),
            AnyId::Range(_)
        ));
        assert!(matches!(
            "pkg:com:name@<1.0.0,>=0.1.0".parse::<AnyId>().unwrap(),
            AnyId::Range(_)
        ));
    }
}

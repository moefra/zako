use semver::Version;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;
use unicode_ident::{is_xid_continue, is_xid_start};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ident {
    ident: String,
}

impl Ident {
    pub type Err = IdentError;

    pub fn from(ident: &str) -> Result<Self, Self::Err> {
        if ident.is_empty() {
            return Err(Self::Err::EmptyString());
        }

        let mut iter = ident.chars();

        let first = iter.next().unwrap();

        if !is_xid_start(first) {
            return Err(Self::Err::XidStartFailed(first));
        }

        for following in iter {
            if !is_xid_continue(following) {
                return Err(Self::Err::XidContinueFailed(following));
            }
        }

        Ok(Self {
            ident: ident.to_string(),
        })
    }

    pub fn ident(&self) -> &str {
        &self.ident
    }
}

#[derive(Error, Debug)]
pub enum IdentError {
    #[error("the ident string is empty")]
    EmptyString(),
    #[error("is_xid_start() of first character `{0}` returns false")]
    XidStartFailed(char),
    #[error("is_xid_continue() of following character `{0}` returns false")]
    XidContinueFailed(char),
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ident)
    }
}

impl FromStr for Ident {
    type Err = IdentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from(s)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GroupId {
    group_id: Vec<Ident>,
}

#[derive(Error, Debug)]
pub enum GroupIdError {
    #[error("GroupId::from() must input a vec that has item")]
    EmptyIdentVec(),
    #[error("GroupId::from_str() has a bad input `{0}`")]
    BadFormat(String),
    #[error("failed to parse identifier")]
    InvalidPart(#[from] Box<dyn std::error::Error>),
}

impl fmt::Display for GroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(first) = self.group_id.first() {
            write!(f, "{}", first)?;
            for item in self.group_id.iter().skip(1) {
                write!(f, ".{}", item)?;
            }
        }
        Ok(())
    }
}

impl FromStr for GroupId {
    type Err = GroupIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut idents = Vec::<Ident>::new();

        for ident_str in s.split('.') {
            let result =
                Ident::from_str(ident_str).map_err(|err| Self::Err::InvalidPart(err.into()))?;
            idents.push(result);
        }

        Self::from(idents)
    }
}

impl GroupId {
    pub type Err = GroupIdError;

    pub fn from(idents: Vec<Ident>) -> Result<GroupId, GroupIdError> {
        if idents.is_empty() {
            return Err(GroupIdError::EmptyIdentVec());
        }
        Ok(GroupId { group_id: idents })
    }

    pub fn get_id(&self) -> &Vec<Ident> {
        &self.group_id
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ArtifactId {
    group_id: GroupId,
    artifact_id: Ident,
}

impl fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.group_id, self.artifact_id)
    }
}

impl FromStr for ArtifactId {
    type Err = ArtifactIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split(':');

        let group_id_str = iter.next();

        let artifact_id_str = iter.next();

        if group_id_str.is_none() || artifact_id_str.is_none() || iter.next().is_some() {
            return Err(Self::Err::BadFormat(s.to_string()));
        }

        let group_id_str = group_id_str.unwrap();
        let artifact_id_str = artifact_id_str.unwrap();

        let group_id =
            GroupId::from_str(group_id_str).map_err(|err| Self::Err::InvalidPart(err.into()))?;

        let artifact_id =
            Ident::from_str(artifact_id_str).map_err(|err| Self::Err::InvalidPart(err.into()))?;

        Ok(Self::from(group_id, artifact_id))
    }
}

#[derive(Error, Debug)]
pub enum ArtifactIdError {
    #[error("ArtifactId::from_str() has a bad input `{0}`")]
    BadFormat(String),
    #[error("failed to parse some part")]
    InvalidPart(#[from] Box<dyn std::error::Error>),
}

impl ArtifactId {
    pub type Err = ArtifactIdError;

    pub fn from(group_id: GroupId, artifact_id: Ident) -> ArtifactId {
        ArtifactId {
            group_id,
            artifact_id,
        }
    }

    pub fn get_group_id(&self) -> &GroupId {
        &self.group_id
    }

    pub fn get_artifact_id(&self) -> &Ident {
        &self.artifact_id
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QualifiedArtifactId {
    artifact_id: ArtifactId,
    version: Version,
}

#[derive(Error, Debug)]
pub enum QualifiedArtifactIdError {
    #[error("QualifiedArtifactId::from_str() has a bad input `{0}`")]
    BadFormat(String),
    #[error("failed to parse some part")]
    InvalidPart(#[from] Box<dyn std::error::Error>),
}

impl fmt::Display for QualifiedArtifactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.artifact_id, self.version)
    }
}

impl FromStr for QualifiedArtifactId {
    type Err = QualifiedArtifactIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (artifact_part, version_part) = s
            .rsplit_once('@')
            .ok_or_else(|| QualifiedArtifactIdError::BadFormat(s.to_string()))?;

        let artifact_id = ArtifactId::from_str(artifact_part)
            .map_err(|err| QualifiedArtifactIdError::InvalidPart(err.into()))?;
        let version = Version::from_str(version_part)
            .map_err(|err| QualifiedArtifactIdError::InvalidPart(err.into()))?;

        Ok(Self::from(artifact_id, version))
    }
}

impl QualifiedArtifactId {
    pub type Err = QualifiedArtifactIdError;

    pub fn from(artifact_id: ArtifactId, version: Version) -> QualifiedArtifactId {
        QualifiedArtifactId {
            artifact_id,
            version,
        }
    }
    pub fn get_artifact_id(&self) -> &ArtifactId {
        &self.artifact_id
    }
    pub fn get_version(&self) -> &Version {
        &self.version
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Id {
    artifact_id: QualifiedArtifactId,
    name: Vec<Ident>,
}

impl Id {
    pub type Err = IdError;

    pub fn from(artifact_id: QualifiedArtifactId, name: Vec<Ident>) -> Result<Id, IdError> {
        if name.is_empty() {
            return Err(IdError::EmptyIdentVec());
        }
        Ok(Id { artifact_id, name })
    }
    pub fn get_artifact_id(&self) -> &QualifiedArtifactId {
        &self.artifact_id
    }
    pub fn get_name(&self) -> &Vec<Ident> {
        &self.name
    }
}

impl FromStr for Id {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (qualified_artifact_id,name) =
            s.rsplit_once('#')
            .ok_or_else(|| IdError::BadFormat(s.to_string()))?;

        let artifact =
            QualifiedArtifactId::from_str(qualified_artifact_id)
            .map_err(|err| Self::Err::InvalidPart(err.into()))?;

        let mut idents = Vec::<Ident>::new();

        for ident in name.split('.') {
            let result =
                Ident::from_str(ident).map_err(|err| Self::Err::InvalidPart(err.into()))?;
            idents.push(result);
        }

        Self::from(artifact, idents)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#", self.artifact_id)?;
        if let Some(first) = self.name.first() {
            write!(f, "{}", first)?;
            for item in self.name.iter().skip(1) {
                write!(f, ".{}", item)?;
            }
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum IdError {
    #[error("Id::from() must input a vec that has item")]
    EmptyIdentVec(),
    #[error("Id::from_str() has a bad input `{0}`")]
    BadFormat(String),
    #[error("failed to parse some part")]
    InvalidPart(#[from] Box<dyn std::error::Error>),
}

/// The type of target,like Build, Test or Install.
#[derive(Debug,Clone,Eq, PartialEq)]
pub struct TargetType(pub Id);
/// The computer architecture, like arm64
#[derive(Debug,Clone,Eq, PartialEq)]
pub struct Architecture(pub Id);
/// The operating system,like linux.
#[derive(Debug,Clone,Eq, PartialEq)]
pub struct Os(pub Id);
/// The type of tool,like c compiler.
#[derive(Debug,Clone,Eq, PartialEq)]
pub struct ToolType(pub Id);
/// The name of tool,like gcc or clang.
#[derive(Debug,Clone,Eq, PartialEq)]
pub struct ToolName(pub Id);

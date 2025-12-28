use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use ts_rs::TS;
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    intern::{InternedAbsolutePath, Interner},
    package_id::{InternedPackageId, PackageIdParseError},
};

#[derive(thiserror::Error, Debug)]
pub enum PackageSourceResolveError {
    #[error("failed to parse package")]
    FailedToResolve(#[from] PackageIdParseError),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
    #[error("Interner error while processing package source: {0}")]
    InternerError(#[from] ::zako_interner::InternerError),
    #[error("the path `{0}` is not an absolute path")]
    PathNotAbsolute(String),
}

#[derive(
    TS,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Hash,
    PartialEq,
    Eq,
    rkyv::Deserialize,
    rkyv::Serialize,
    rkyv::Archive,
)]
#[ts(export, export_to = "dependency_source.d.ts")]
#[ts(optional_fields)]
#[serde(untagged)]
/// The source of a package.
///
/// Its path should be relative path, relative to the project root.
///
/// Use it to calculate hash, not [ResolvedPackageSource].
pub enum PackageSource {
    /// 来源于远程仓库
    Registry { package: String },
    /// 来源于Git仓库
    Git {
        #[ts(as = "::std::string::String")]
        /// Url of the repo
        repo: SmolStr,
        #[ts(as = "::std::option::Option<::std::string::String>")]
        checkout: Option<SmolStr>,
    },
    /// 来源于HTTP下载
    Http {
        #[ts(as = "::std::string::String")]
        url: SmolStr,
    },
    /// 来源于本地路径
    Path { path: String },
}

impl Blake3Hash for PackageSource {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        match self {
            PackageSource::Registry { package } => package.hash_into_blake3(hasher),
            PackageSource::Git { repo, checkout } => {
                repo.hash_into_blake3(hasher);
                checkout.hash_into_blake3(hasher);
            }
            PackageSource::Http { url } => url.hash_into_blake3(hasher),
            PackageSource::Path { path } => path.hash_into_blake3(hasher),
        }
    }
}

/// The resolved source of a package.
///
/// Its path should be absolute path, absolute to the project root.
///
/// Do not use it to calculate hash, use [PackageSource] instead.
#[derive(Debug, Clone, Hash, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub enum ResolvedPackageSource {
    Registry {
        package: InternedPackageId,
    },
    Git {
        repo: SmolStr,
        checkout: Option<SmolStr>,
    },
    Http {
        url: SmolStr,
    },
    Path {
        path: InternedAbsolutePath,
    },
}

impl PackageSource {
    pub fn resolve(
        self,
        _current_path: &Utf8Path,
        interner: &Interner,
    ) -> Result<ResolvedPackageSource, PackageSourceResolveError> {
        match self {
            PackageSource::Registry { package } => {
                let interned_package = InternedPackageId::try_parse(&package, interner)?;
                Ok(ResolvedPackageSource::Registry {
                    package: interned_package,
                })
            }
            PackageSource::Git { repo, checkout } => {
                Ok(ResolvedPackageSource::Git { repo, checkout })
            }
            PackageSource::Http { url } => Ok(ResolvedPackageSource::Http { url }),
            PackageSource::Path { path } => Ok(ResolvedPackageSource::Path {
                path: InternedAbsolutePath::new(path.as_str(), interner)?
                    .ok_or_else(|| PackageSourceResolveError::PathNotAbsolute(path.clone()))?,
            }),
        }
    }
}

impl ResolvedPackageSource {
    pub fn to_raw(&self, interner: &Interner) -> Result<PackageSource, PackageSourceResolveError> {
        match self {
            ResolvedPackageSource::Registry { package } => Ok(PackageSource::Registry {
                package: format!(
                    "{}:{}@{}",
                    interner.resolve(&package.name.group.0)?,
                    interner.resolve(&package.name.name.0)?,
                    interner.resolve(&package.version.0)?
                ),
            }),
            ResolvedPackageSource::Git { repo, checkout } => Ok(PackageSource::Git {
                repo: repo.clone(),
                checkout: checkout.clone(),
            }),
            ResolvedPackageSource::Http { url } => Ok(PackageSource::Http { url: url.clone() }),
            ResolvedPackageSource::Path { path } => Ok(PackageSource::Path {
                path: interner.resolve(path.interned)?.into(),
            }),
        }
    }
}

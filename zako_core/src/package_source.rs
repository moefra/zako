use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use ts_rs::TS;
use zako_digest::blake3::Blake3Hash;

use crate::{
    intern::{Internable, InternedAbsolutePath, Interner, Uninternable},
    package_id::{InternedPackageId, PackageIdParseError},
    path::{NeutralPath, PathError, interned::InternedNeutralPath},
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
        /// Url of the repo, must not be `/file/path/to/local/or/file/url/protocol`, use [PackageSource::Path] instead.
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
    ///
    /// The path must be relative and relative to the project root.
    Path { path: String },
}

impl PackageSource {
    pub fn validate(&self) -> eyre::Result<()> {
        match self {
            PackageSource::Path { path } => {
                let p = NeutralPath::from_path(path)?;
                if !p.is_in_dir(NeutralPath::dot()) {
                    return Err(eyre::eyre!(
                        "the path `{}` is not relative to the project root",
                        path
                    ));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
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

/// The interned and **checked** [PackageSource].
///
/// The `checked` means that the path is relative to the project root and will not access outside of the project root.
#[derive(Debug, Clone, Hash, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub enum InternedPackageSource {
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
        path: InternedNeutralPath,
    },
}

impl Internable for PackageSource {
    type Interned = InternedPackageSource;

    fn intern(self, interner: &Interner) -> eyre::Result<InternedPackageSource> {
        self.validate()?;

        match self {
            PackageSource::Registry { package } => {
                let interned_package = InternedPackageId::try_parse(&package, interner)?;
                Ok(InternedPackageSource::Registry {
                    package: interned_package,
                })
            }
            PackageSource::Git { repo, checkout } => {
                Ok(InternedPackageSource::Git { repo, checkout })
            }
            PackageSource::Http { url } => Ok(InternedPackageSource::Http { url }),
            PackageSource::Path { path } => {
                let path = NeutralPath::from_path(path.as_str())?;

                Ok(InternedPackageSource::Path {
                    path: path.intern(interner)?,
                })
            }
        }
    }
}

impl Uninternable for InternedPackageSource {
    type Uninterned = PackageSource;

    fn unintern(&self, interner: &Interner) -> eyre::Result<Self::Uninterned> {
        let interner = interner.as_ref();
        match self {
            InternedPackageSource::Registry { package } => Ok(PackageSource::Registry {
                package: format!(
                    "{}:{}@{}",
                    interner.resolve(&package.name.group)?,
                    interner.resolve(&package.name.name)?,
                    interner.resolve(&package.version)?
                ),
            }),
            InternedPackageSource::Git { repo, checkout } => Ok(PackageSource::Git {
                repo: repo.clone(),
                checkout: checkout.clone(),
            }),
            InternedPackageSource::Http { url } => Ok(PackageSource::Http { url: url.clone() }),
            InternedPackageSource::Path { path } => Ok(PackageSource::Path {
                path: interner.resolve(path)?.into(),
            }),
        }
    }
}

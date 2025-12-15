use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use ts_rs::TS;

use crate::{
    intern::{InternedString, Interner},
    package::{InternedPackage, PackageParseError},
};

#[derive(thiserror::Error, Debug)]
pub enum PackageSourceResolveError {
    #[error("failed to parse package")]
    FailedToResolve(#[from] PackageParseError),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
}

#[derive(TS, Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[ts(export, export_to = "dependency_source.d.ts")]
#[ts(optional_fields)]
#[serde(untagged)]
/// 一个包的来源
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ResolvedPackageSource {
    Registry {
        package: InternedPackage,
    },
    Git {
        repo: SmolStr,
        checkout: Option<SmolStr>,
    },
    Http {
        url: SmolStr,
    },
    Path {
        path: SmolStr,
    },
}

impl PackageSource {
    pub fn resolve(
        self,
        current_path: &PathBuf,
        interner: &Interner,
    ) -> Result<ResolvedPackageSource, PackageSourceResolveError> {
        match self {
            PackageSource::Registry { package } => {
                let interned_package = InternedPackage::try_parse(&package, interner)?;
                Ok(ResolvedPackageSource::Registry {
                    package: interned_package,
                })
            }
            PackageSource::Git { repo, checkout } => {
                Ok(ResolvedPackageSource::Git { repo, checkout })
            }
            PackageSource::Http { url } => Ok(ResolvedPackageSource::Http { url }),
            PackageSource::Path { path } => {
                let pathbuf = PathBuf::from(path.as_str());
                let resolved_path = if pathbuf.is_absolute() {
                    SmolStr::new(path)
                } else {
                    SmolStr::new(current_path.join(pathbuf).canonicalize()?.to_string_lossy())
                };
                Ok(ResolvedPackageSource::Path {
                    path: SmolStr::new(resolved_path),
                })
            }
        }
    }
}

impl ResolvedPackageSource {
    pub fn to_raw(&self, interner: &Interner) -> PackageSource {
        match self {
            ResolvedPackageSource::Registry { package } => PackageSource::Registry {
                package: format!(
                    "{}:{}@{}",
                    interner.resolve(&package.name.group.0),
                    interner.resolve(&package.name.name.0),
                    interner.resolve(&package.version.0)
                ),
            },
            ResolvedPackageSource::Git { repo, checkout } => PackageSource::Git {
                repo: repo.clone(),
                checkout: checkout.clone(),
            },
            ResolvedPackageSource::Http { url } => PackageSource::Http { url: url.clone() },
            ResolvedPackageSource::Path { path } => PackageSource::Path {
                path: path.to_string(),
            },
        }
    }
}

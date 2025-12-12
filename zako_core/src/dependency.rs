use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::id::InternedString;

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
        repo: String,
        checkout: Option<String>,
    },
    /// 来源于HTTP下载
    Http { url: String },
    /// 来源于本地路径
    Path { path: String },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum InternedPackageSource {
    Registry {
        package: InternedString,
    },
    Git {
        repo: InternedString,
        checkout: Option<InternedString>,
    },
    Http {
        url: InternedString,
    },
    Path {
        path: InternedString,
    },
}

impl InternedPackageSource {
    pub fn from_raw(source: &PackageSource, interner: &mut crate::id::Interner) -> Self {
        match source {
            PackageSource::Registry { package } => InternedPackageSource::Registry {
                package: interner.get_or_intern(&package),
            },
            PackageSource::Git { repo, checkout } => InternedPackageSource::Git {
                repo: interner.get_or_intern(&repo),
                checkout: checkout.as_ref().map(|c| interner.get_or_intern(&c)),
            },
            PackageSource::Http { url } => InternedPackageSource::Http {
                url: interner.get_or_intern(&url),
            },
            PackageSource::Path { path } => InternedPackageSource::Path {
                path: interner.get_or_intern(&path),
            },
        }
    }

    pub fn to_raw(&self, interner: &crate::id::Interner) -> PackageSource {
        match self {
            InternedPackageSource::Registry { package } => PackageSource::Registry {
                package: interner.resolve(&package).to_string(),
            },
            InternedPackageSource::Git { repo, checkout } => PackageSource::Git {
                repo: interner.resolve(&repo).to_string(),
                checkout: checkout.map(|c| interner.resolve(&c).to_string()),
            },
            InternedPackageSource::Http { url } => PackageSource::Http {
                url: interner.resolve(&url).to_string(),
            },
            InternedPackageSource::Path { path } => PackageSource::Path {
                path: interner.resolve(&path).to_string(),
            },
        }
    }
}

#[derive(TS, Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[ts(export, export_to = "dependency.d.ts")]
#[ts(optional_fields)]
#[serde(untagged)]
pub enum Dependency {
    /// alias of Source(PackageSource::Registry)
    Package(String),
    ComplexPackage {
        /// package name without version
        source: String,
        version: String,
        optional: bool,
    },
    Source(PackageSource),
}

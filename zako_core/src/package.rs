use std::collections::{BTreeMap, HashMap};

use crate::intern::{Internable, Uninternable};
use crate::pattern::{InternedPattern, Pattern, PatternError, PatternGroup};
use crate::{
    author::{Author, InternedAuthor},
    config_value::ConfigValue,
    context::BuildContext,
    intern::InternedString,
    package_id::{InternedGroup, InternedVersion},
    package_source::PackageSource,
};
use crate::{
    config::{Configuration, ResolvedConfiguration},
    package_id::InternedArtifactId,
};
use crate::{id::InternedAtom, package_id::InternedPackageId};
use crate::{intern::Interner, package_source::InternedPackageSource};
use ::camino::Utf8PathBuf;
use ::zako_interner::InternerError;
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use ts_rs::TS;
use zako_digest::blake3::{Blake3Hash, Hash};

#[derive(thiserror::Error, Debug)]
pub enum PackageResolveError {
    #[error("the package dependecies key `{0}` is not a valid xid_loose_ident")]
    InvalidDependencyKey(String),
    #[error("the package config key `{0}` is not a valid xid_loose_ident")]
    InvalidConfigKey(String),
    #[error("failed to parse package id of package: {0}")]
    PackageParseError(#[from] crate::package_id::PackageIdParseError),
    #[error("failed to resolve package source of the package: {0}")]
    PackageSourceResolveError(#[from] crate::package_source::PackageSourceResolveError),
    #[error("failed to parse the id `{0}` of package part `{1}`: {2}")]
    IdParseError(String, &'static str, #[source] crate::id::IdParseError),
    #[error("failed to resolve configuration of the package: {0}")]
    ConfigResolveError(#[source] eyre::Report),
    #[error("author resolution error: {0}")]
    AuthorError(#[from] crate::author::AuthorError),
    #[error("pattern resolution error: {0}")]
    PatternError(#[from] crate::pattern::PatternError),
    #[error("configuration resolution error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
    #[error("interner error: {0}")]
    InternerError(#[from] ::zako_interner::InternerError),
    #[error("other error: {0}")]
    OtherError(#[from] eyre::Report),
}

#[derive(
    TS,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    rkyv::Deserialize,
    rkyv::Serialize,
    rkyv::Archive,
)]
#[ts(export, export_to = "package.d.ts")]
#[ts(optional_fields)]
pub struct Package {
    pub group: SmolStr,
    pub artifact: SmolStr,
    pub version: SmolStr,
    pub configure_script: Option<SmolStr>,
    pub description: Option<SmolStr>,
    pub authors: Option<Vec<Author>>,
    pub license: Option<SmolStr>,
    pub builds: Option<Pattern>,
    pub rules: Option<Pattern>,
    pub toolchains: Option<Pattern>,
    pub peers: Option<Pattern>,
    /// The key will be checked by [crate::id::is_loose_ident]
    pub dependencies: Option<BTreeMap<SmolStr, PackageSource>>,
    /// Default mount config to [crate::consts::DEFAULT_CONFIGURATION_MOUNT_POINT]
    pub mount_config: Option<SmolStr>,
    /// The key will be checked by [crate::id::is_loose_ident]
    pub config: Option<BTreeMap<SmolStr, ConfigValue>>,
}

impl Package {
    #[must_use]
    pub fn validate(&self) -> Result<(), PackageResolveError> {
        if let Some(wrong_config_key) = self.config.as_ref().and_then(|cfg| {
            cfg.keys().find(|k| {
                let parsed = crate::id::is_loose_ident(k);
                parsed == false
            })
        }) {
            return Err(PackageResolveError::InvalidConfigKey(
                wrong_config_key.to_string(),
            ));
        }

        if let Some(wrong_dependency_key) = self.dependencies.as_ref().and_then(|deps| {
            deps.keys().find(|k| {
                let parsed = crate::id::is_loose_ident(k);
                parsed == false
            })
        }) {
            return Err(PackageResolveError::InvalidDependencyKey(
                wrong_dependency_key.to_string(),
            ));
        }

        Ok(())
    }
}

impl Blake3Hash for Package {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.group.hash_into_blake3(hasher);
        self.artifact.hash_into_blake3(hasher);
        self.version.hash_into_blake3(hasher);
        self.configure_script.hash_into_blake3(hasher);
        self.description.hash_into_blake3(hasher);
        self.authors.hash_into_blake3(hasher);
        self.license.hash_into_blake3(hasher);
        self.builds.hash_into_blake3(hasher);
        self.rules.hash_into_blake3(hasher);
        self.toolchains.hash_into_blake3(hasher);
        self.peers.hash_into_blake3(hasher);
        self.dependencies.hash_into_blake3(hasher);
        self.mount_config.hash_into_blake3(hasher);
        self.config.hash_into_blake3(hasher);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolvingPackage {
    pub original: Package,
    pub resolved_config: ResolvedConfiguration,
    pub additional_peers: Vec<Pattern>,
    pub additional_builds: Vec<Pattern>,
    pub additional_rules: Vec<Pattern>,
    pub additional_toolchains: Vec<Pattern>,
}

impl ResolvingPackage {
    pub fn new(original: Package, resolved_config: ResolvedConfiguration) -> Self {
        Self {
            original,
            resolved_config,
            additional_peers: Default::default(),
            additional_builds: Default::default(),
            additional_rules: Default::default(),
            additional_toolchains: Default::default(),
        }
    }

    pub fn get_blake3(&self, interner: &Interner) -> eyre::Result<Hash> {
        let mut hasher = blake3::Hasher::new();

        self.original.hash_into_blake3(&mut hasher);
        self.resolved_config
            .resolve(interner)?
            .hash_into_blake3(&mut hasher);
        self.additional_peers.hash_into_blake3(&mut hasher);
        self.additional_builds.hash_into_blake3(&mut hasher);
        self.additional_rules.hash_into_blake3(&mut hasher);
        self.additional_toolchains.hash_into_blake3(&mut hasher);

        Ok(hasher.finalize().into())
    }

    #[must_use]
    pub fn resolve(self, interner: &Interner) -> Result<ResolvedPackage, PackageResolveError> {
        let trans = |pattern: Option<Pattern>,
                     additional: Vec<Pattern>|
         -> Result<PatternGroup, PatternError> {
            let mut results = Vec::with_capacity(additional.len() + 1);

            if let Some(pattern) = pattern {
                results.push(pattern);
            }

            results.extend(additional);

            Ok(PatternGroup::new(results, interner)?)
        };

        Ok(ResolvedPackage {
            group: InternedGroup::try_parse(&self.original.group, interner)?,
            artifact: InternedAtom::try_parse(&self.original.artifact, interner).map_err(
                |err| {
                    PackageResolveError::IdParseError(
                        self.original.artifact.to_string(),
                        "artifact",
                        err,
                    )
                },
            )?,
            version: InternedVersion::try_parse(&self.original.version, interner)?,
            description: self.original.description,
            authors: self.original.authors.intern(interner)?,
            license: self
                .original
                .license
                .as_ref()
                .map(|s| interner.get_or_intern(s))
                .transpose()?,
            builds: trans(self.original.builds, self.additional_builds)?,
            configure_script: self.original.configure_script,
            rules: trans(self.original.rules, self.additional_rules)?,
            toolchains: trans(self.original.toolchains, self.additional_toolchains)?,
            peers: trans(self.original.peers, self.additional_peers)?,
            dependencies: self
                .original
                .dependencies
                .map(|deps| {
                    deps.into_iter()
                        .map(|(k, v)| v.intern(interner).map(|resolved| (k, resolved)))
                        .collect::<Result<BTreeMap<_, _>, _>>()
                })
                .transpose()?,
            mount_config: self
                .original
                .mount_config
                .as_ref()
                .map(|s| {
                    InternedAtom::try_parse(s, interner).map_err(|err| {
                        PackageResolveError::IdParseError(s.to_string(), "mount_config", err)
                    })
                })
                .transpose()?,
            config: self.resolved_config,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolvedPackage {
    pub group: InternedGroup,
    pub artifact: InternedAtom,
    pub version: InternedVersion,
    pub configure_script: Option<SmolStr>,
    pub description: Option<SmolStr>,
    pub authors: Option<Vec<InternedAuthor>>,
    pub license: Option<InternedString>,
    pub builds: PatternGroup,
    pub rules: PatternGroup,
    pub toolchains: PatternGroup,
    pub peers: PatternGroup,
    pub dependencies: Option<BTreeMap<SmolStr, InternedPackageSource>>,
    pub mount_config: Option<InternedAtom>,
    pub config: ResolvedConfiguration,
}

impl ResolvedPackage {
    pub fn get_artifact_id(&self) -> InternedArtifactId {
        InternedArtifactId {
            group: self.group.clone(),
            name: self.artifact.clone(),
        }
    }

    pub fn get_id(&self) -> InternedPackageId {
        InternedPackageId::new(self.get_artifact_id(), self.version)
    }

    pub fn get_blake3_hash(&self, interner: &Interner) -> eyre::Result<Hash> {
        let mut hasher = blake3::Hasher::new();

        hasher.update(self.group.unintern(interner)?.as_bytes());
        hasher.update(self.artifact.unintern(interner)?.as_bytes());
        hasher.update(self.version.unintern(interner)?.as_bytes());
        hasher.update(
            self.configure_script
                .as_ref()
                .map(|s| s.as_bytes())
                .unwrap_or_default(),
        );
        hasher.update(
            self.description
                .as_ref()
                .map(|s| s.as_bytes())
                .unwrap_or_default(),
        );
        self.authors
            .as_ref()
            .map(|a: &Vec<InternedAuthor>| a.iter().map(|a| a.unintern(interner)).collect())
            .transpose()?
            .map(|mut a: Vec<Author>| {
                a.sort();

                a.iter().for_each(|a| {
                    a.hash_into_blake3(&mut hasher);
                })
            })
            .unwrap_or_else(|| ().hash_into_blake3(&mut hasher));

        if let Some(license) = self
            .license
            .as_ref()
            .map(|l| l.unintern(interner))
            .transpose()?
        {
            hasher.update(license.as_bytes());
        } else {
            ().hash_into_blake3(&mut hasher);
        }

        let mut compute_hash = |pattern_group: &PatternGroup| -> eyre::Result<()> {
            for pattern in pattern_group.patterns.iter() {
                let pattern = pattern.unintern(interner)?;

                pattern.hash_into_blake3(&mut hasher);
            }
            Ok(())
        };
        compute_hash(&self.builds)?;
        compute_hash(&self.rules)?;
        compute_hash(&self.toolchains)?;
        compute_hash(&self.peers)?;

        if let Some(dependencies) = &self.dependencies {
            hasher.update(&dependencies.len().to_le_bytes());
            for (key, value) in dependencies {
                hasher.update(key.as_bytes());
                value.unintern(interner)?.hash_into_blake3(&mut hasher);
            }
        } else {
            ().hash_into_blake3(&mut hasher);
        }

        if let Some(mount_config) = self
            .mount_config
            .as_ref()
            .map(|s| s.unintern(interner))
            .transpose()?
        {
            hasher.update(mount_config.as_bytes());
        } else {
            ().hash_into_blake3(&mut hasher);
        }

        let config: BTreeMap<_, _> = self.config.resolve(interner)?.config;

        for (key, value) in config {
            hasher.update(key.as_bytes());
            value.hash_into_blake3(&mut hasher);
        }

        Ok(hasher.finalize().into())
    }
}

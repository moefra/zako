use std::collections::HashMap;

use crate::package_source::ResolvedPackageSource;
use crate::pattern::{InternedPattern, Pattern};
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
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use ts_rs::TS;
use zako_digest::blake3_hash::Blake3Hash;

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
    #[error("Author resolution error: {0}")]
    AuthorError(#[from] crate::author::AuthorError),
    #[error("Pattern resolution error: {0}")]
    PatternError(#[from] crate::pattern::PatternError),
    #[error("Config resolution error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
    #[error("Interner error: {0}")]
    InternerError(#[from] ::zako_interner::InternerError),
}

#[derive(
    TS,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
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
    /// The key will be checked by [crate::id::is_xid_loose_ident]
    pub dependencies: Option<HashMap<SmolStr, PackageSource>>,
    /// Default mount config to [crate::consts::DEFAULT_CONFIGURATION_MOUNT_POINT]
    pub mount_config: Option<SmolStr>,
    /// The key will be checked by [crate::id::is_xid_loose_ident]
    pub config: Option<HashMap<SmolStr, ConfigValue, ahash::RandomState>>,
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

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolvedPackage {
    pub group: InternedGroup,
    pub artifact: InternedAtom,
    pub version: InternedVersion,
    pub configure_script: Option<SmolStr>,
    pub description: Option<SmolStr>,
    pub authors: Option<Vec<InternedAuthor>>,
    pub license: Option<InternedString>,
    pub builds: Option<InternedPattern>,
    pub rules: Option<InternedPattern>,
    pub toolchains: Option<InternedPattern>,
    pub peers: Option<InternedPattern>,
    pub dependencies: Option<HashMap<SmolStr, ResolvedPackageSource>>,
    pub mount_config: Option<InternedAtom>,
    pub config: ResolvedConfiguration,
}

impl Package {
    #[must_use]
    pub fn resolve(
        self,
        context: &BuildContext,
        project_root: &Utf8Path,
    ) -> Result<ResolvedPackage, PackageResolveError> {
        let dbg_msg = format!("while resolving project {:?}", &self);

        let authors = if let Some(authors) = self.authors {
            let mut interned_authors: Vec<InternedAuthor> = Vec::with_capacity(authors.len());
            for author in authors.into_iter() {
                interned_authors.push(author.intern(context)?);
            }
            Some(interned_authors)
        } else {
            None
        };

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

        Ok(ResolvedPackage {
            group: InternedGroup::try_parse(&self.group, context.interner())?,
            artifact: InternedAtom::try_parse(&self.artifact, context.interner()).map_err(
                |err| PackageResolveError::IdParseError(self.artifact.to_string(), "artifact", err),
            )?,
            version: InternedVersion::try_parse(&self.version, context.interner())?,
            description: self.description,
            authors,
            license: self
                .license
                .as_ref()
                .map(|s| context.interner().get_or_intern(s))
                .transpose()?,
            builds: self
                .builds
                .map(|pattern| pattern.intern(context))
                .transpose()?,
            configure_script: self.configure_script,
            rules: self
                .rules
                .map(|pattern| pattern.intern(context))
                .transpose()?,
            toolchains: self
                .toolchains
                .map(|pattern| pattern.intern(context))
                .transpose()?,
            peers: self
                .peers
                .map(|pattern| pattern.intern(context))
                .transpose()?,
            dependencies: self
                .dependencies
                .map(|deps| {
                    deps.into_iter()
                        .map(|(k, v)| {
                            v.resolve(project_root, context.interner())
                                .map(|resolved| (k, resolved))
                        })
                        .collect::<Result<HashMap<_, _>, _>>()
                })
                .transpose()?,
            mount_config: self
                .mount_config
                .as_ref()
                .map(|s| {
                    InternedAtom::try_parse(s, context.interner()).map_err(|err| {
                        PackageResolveError::IdParseError(s.to_string(), "mount_config", err)
                    })
                })
                .transpose()?,
            config: Configuration::from(self.config.unwrap_or_default())
                .resolve(context.interner())
                .map_err(|err| PackageResolveError::ConfigResolveError(err.wrap_err(dbg_msg)))?,
        })
    }
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

    pub fn to_raw(&self, context: &BuildContext) -> Result<Package, PackageResolveError> {
        let interner = context.interner();
        Ok(Package {
            group: SmolStr::new(context.interner().resolve(&self.group.0)?),
            artifact: SmolStr::new(context.interner().resolve(&self.artifact.0)?),
            version: SmolStr::new(context.interner().resolve(&self.version.0)?),
            description: self.description.clone(),
            authors: self
                .authors
                .as_ref()
                .map(|v| {
                    v.iter()
                        .map(|a| InternedAuthor::resolve(a, context))
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?,
            configure_script: self.configure_script.clone(),
            license: self
                .license
                .as_ref()
                .map(|s| {
                    context
                        .interner()
                        .resolve(&s)
                        .map(|string| SmolStr::new(string))
                })
                .transpose()?,
            builds: self
                .builds
                .as_ref()
                .map(|p| InternedPattern::resolve(p, interner))
                .transpose()?,
            rules: self
                .rules
                .as_ref()
                .map(|p| InternedPattern::resolve(p, interner))
                .transpose()?,
            toolchains: self
                .toolchains
                .as_ref()
                .map(|p| InternedPattern::resolve(p, interner))
                .transpose()?,
            peers: self
                .peers
                .as_ref()
                .map(|p| InternedPattern::resolve(p, interner))
                .transpose()?,
            dependencies: self
                .dependencies
                .as_ref()
                .map(|deps| {
                    deps.iter()
                        .map(|(k, v)| Ok((k.clone(), v.to_raw(context.interner())?)))
                        .collect::<Result<HashMap<_, _>, PackageResolveError>>()
                })
                .transpose()?,
            mount_config: self
                .mount_config
                .map(|s| {
                    context
                        .interner()
                        .resolve(&s.0)
                        .map(|string| SmolStr::new(string))
                })
                .transpose()?,
            config: Some(self.config.resolve(context.interner())?.config),
        })
    }
}

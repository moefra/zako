use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::id::{InternedAtom, InternedLabel};
use crate::package_source::ResolvedPackageSource;
use crate::pattern::{InternedPattern, Pattern};
use crate::{
    author::{Author, InternedAuthor},
    config::Config,
    context::BuildContext,
    intern::{Internable, InternedString},
    package::{InternedGroup, InternedVersion},
    package_source::PackageSource,
};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use ts_rs::TS;

#[derive(thiserror::Error, Debug)]
pub enum ProjectResolveError {
    #[error("the project dependecies key `{0}` is not a valid xid_loose_ident")]
    InvalidDependencyKey(String),
    #[error("the project config key `{0}` is not a valid xid_loose_ident")]
    InvalidConfigKey(String),
    #[error("failed to parse package id of project: {0}")]
    PackageParseError(#[from] crate::package::PackageParseError),
    #[error("failed to resolve package source of the project: {0}")]
    PackageSourceResolveError(#[from] crate::package_source::PackageSourceResolveError),
    #[error("failed to parse the id `{0}` of project part `{1}`: {2}")]
    IdParseError(String, &'static str, #[source] crate::id::IdParseError),
}

#[derive(TS, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[ts(export, export_to = "project.d.ts")]
#[ts(optional_fields)]
pub struct Project {
    pub group: String,
    #[ts(as = "::std::option::Option<::std::string::String>")]
    pub artifact: SmolStr,
    pub version: String,
    #[ts(as = "::std::option::Option<::std::string::String>")]
    pub description: Option<SmolStr>,
    pub authors: Option<Vec<Author>>,
    pub license: Option<String>,
    pub builds: Option<Pattern>,
    pub rules: Option<Pattern>,
    pub toolchains: Option<Pattern>,
    pub subprojects: Option<Pattern>,
    /// The key will be checked by [crate::id::is_xid_loose_ident]
    #[ts(as = "::std::option::Option<::std::collections::HashMap<::std::string::String, Config>>")]
    pub dependencies: Option<HashMap<SmolStr, PackageSource>>,
    /// Default mount config to `config`
    pub mount_config: Option<String>,
    /// The key will be checked by [crate::id::is_xid_loose_ident]
    #[ts(as = "::std::option::Option<::std::collections::HashMap<::std::string::String, Config>>")]
    pub config: Option<HashMap<SmolStr, Config>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProject {
    pub group: InternedGroup,
    pub artifact: SmolStr,
    pub version: InternedVersion,
    pub description: Option<SmolStr>,
    pub authors: Option<Vec<InternedAuthor>>,
    pub license: Option<InternedString>,
    pub builds: Option<InternedPattern>,
    pub rules: Option<InternedPattern>,
    pub toolchains: Option<InternedPattern>,
    pub subprojects: Option<InternedPattern>,
    pub dependencies: Option<HashMap<SmolStr, ResolvedPackageSource>>,
    pub mount_config: Option<InternedAtom>,
    pub config: Option<HashMap<SmolStr, Config>>,
}

impl Project {
    pub fn resolve(
        self,
        context: &BuildContext,
        current_path: &PathBuf,
    ) -> Result<ResolvedProject, ProjectResolveError> {
        let authors = if let Some(authors) = self.authors {
            let mut interned_authors: Vec<InternedAuthor> = Vec::with_capacity(authors.len());
            for author in authors.into_iter() {
                interned_authors.push(author.intern(context));
            }
            Some(interned_authors)
        } else {
            None
        };

        if let Some(wrong_dependency_key) = self.dependencies.as_ref().and_then(|deps| {
            deps.keys().find(|k| {
                let parsed = crate::id::is_xid_loose_ident(k);
                parsed == false
            })
        }) {
            return Err(ProjectResolveError::InvalidDependencyKey(
                wrong_dependency_key.to_string(),
            ));
        }

        if let Some(wrong_config_key) = self.config.as_ref().and_then(|cfg| {
            cfg.keys().find(|k| {
                let parsed = crate::id::is_xid_loose_ident(k);
                parsed == false
            })
        }) {
            return Err(ProjectResolveError::InvalidConfigKey(
                wrong_config_key.to_string(),
            ));
        }

        Ok(ResolvedProject {
            group: InternedGroup::try_parse(&self.group, context.interner())?,
            artifact: self.artifact,
            version: InternedVersion::try_parse(&self.version, context.interner())?,
            description: self.description,
            authors,
            license: self
                .license
                .as_ref()
                .map(|s| context.interner().get_or_intern(s)),
            builds: self.builds.map(|pattern| pattern.intern(context)),
            rules: self.rules.map(|pattern| pattern.intern(context)),
            toolchains: self.toolchains.map(|pattern| pattern.intern(context)),
            subprojects: self.subprojects.map(|pattern| pattern.intern(context)),
            dependencies: self
                .dependencies
                .map(|deps| {
                    deps.into_iter()
                        .map(|(k, v)| {
                            v.resolve(current_path, context.interner())
                                .map(|resolved| (k, resolved))
                        })
                        .collect::<Result<HashMap<_, _>, _>>()
                })
                .transpose()?,
            mount_config: self
                .mount_config
                .map(|s| {
                    InternedAtom::try_parse(&s, context.interner()).map_err(|err| {
                        ProjectResolveError::IdParseError(s.clone(), "mount_config", err)
                    })
                })
                .transpose()?,
            config: self
                .config
                .as_ref()
                .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
        })
    }
}

impl ResolvedProject {
    pub fn to_raw(&self, context: &BuildContext) -> Project {
        let interner = context.interner();
        Project {
            group: context.interner().resolve(&self.group.0).to_string(),
            artifact: self.artifact.clone(),
            version: context.interner().resolve(&self.version.0).to_string(),
            description: self.description.clone(),
            authors: self.authors.as_ref().map(|v| {
                v.iter()
                    .map(|a| InternedAuthor::resolve(a, context))
                    .collect()
            }),
            license: self
                .license
                .as_ref()
                .map(|s| context.interner().resolve(&s).to_string()),
            builds: self
                .builds
                .as_ref()
                .map(|p| InternedPattern::resolve(p, interner)),
            rules: self
                .rules
                .as_ref()
                .map(|p| InternedPattern::resolve(p, interner)),
            toolchains: self
                .toolchains
                .as_ref()
                .map(|p| InternedPattern::resolve(p, interner)),
            subprojects: self
                .subprojects
                .as_ref()
                .map(|p| InternedPattern::resolve(p, interner)),
            dependencies: self.dependencies.as_ref().map(|deps| {
                deps.iter()
                    .map(|(k, v)| (k.clone(), v.to_raw(context.interner())))
                    .collect()
            }),
            mount_config: self
                .mount_config
                .map(|s| context.interner().resolve(&s.0).to_string()),
            config: self
                .config
                .as_ref()
                .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
        }
    }
}

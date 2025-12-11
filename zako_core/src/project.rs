use std::{collections::HashMap, path::PathBuf};

use crate::{
    Pattern,
    author::{Author, InternedAuthor},
    config::Config,
    dependency::Dependency,
    id::InternedString,
    package::{InternedArtifactName, InternedGroup, InternedVersion},
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[ts(export, export_to = "project.d.ts")]
#[ts(optional_fields)]
pub struct Project {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<Vec<Author>>,
    pub license: Option<String>,
    pub build: Option<Pattern>,
    pub rule: Option<Pattern>,
    pub toolchain: Option<Pattern>,
    pub subproject: Option<Pattern>,
    pub dependency: Option<HashMap<String, Dependency>>,
    /// Default mount config to `config`
    pub mount_config: Option<String>,
    pub config: Option<HashMap<String, Config>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct InternedProject {
    pub group: InternedGroup,
    pub artifact: InternedArtifactName,
    pub version: InternedVersion,
    pub description: Option<InternedString>,
    pub author: Option<Vec<InternedAuthor>>,
    pub license: Option<InternedString>,
    pub build: Option<Pattern>,
    pub rule: Option<Pattern>,
    pub toolchain: Option<Pattern>,
    pub subproject: Option<Pattern>,
    pub dependency: Option<HashMap<InternedString, Dependency>>,
    pub mount_config: Option<InternedString>,
    pub config: Option<HashMap<InternedString, Config>>,
}

impl InternedProject {
    pub fn from_project(
        project: &Project,
        interner: &mut crate::id::Interner,
    ) -> Result<Self, String> {
        Ok(InternedProject {
            group: InternedGroup::try_parse(&project.group, interner)?,
            artifact: InternedArtifactName::try_parse(&project.artifact, interner)?,
            version: InternedVersion::try_parse(&project.version, interner)?,
            description: project
                .description
                .as_ref()
                .map(|d| interner.get_or_intern(d)),
            author: project
                .author
                .as_ref()
                .map(|authors| authors.iter().map(|a| a.intern(interner)).collect()),
            license: project.license.as_ref().map(|l| interner.get_or_intern(l)),
            build: project.build.clone(),
            rule: project.rule.clone(),
            toolchain: project.toolchain.clone(),
            subproject: project.subproject.clone(),
            dependency: project.dependency.as_ref().map(|deps| {
                deps.iter()
                    .map(|(k, v)| (interner.get_or_intern(k), v.clone()))
                    .collect()
            }),
            mount_config: project
                .mount_config
                .as_ref()
                .map(|m| interner.get_or_intern(m)),
            config: project.config.as_ref().map(|cfg| {
                cfg.iter()
                    .map(|(k, v)| (interner.get_or_intern(k), v.clone()))
                    .collect()
            }),
        })
    }
}

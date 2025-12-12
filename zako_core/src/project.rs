use std::{collections::HashMap, path::PathBuf, sync::Arc};

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
    pub authors: Option<Vec<InternedAuthor>>,
    pub license: Option<InternedString>,
    pub builds: Option<Pattern>,
    pub rules: Option<Pattern>,
    pub toolchains: Option<Pattern>,
    pub subprojects: Option<Pattern>,
    pub dependencies: Option<HashMap<InternedString, Dependency>>,
    pub mount_configuration: Option<InternedString>,
    pub configurations: Option<HashMap<InternedString, Config>>,
}

impl InternedProject {
    pub fn from_raw(project: &Project, interner: &mut crate::id::Interner) -> Result<Self, String> {
        Ok(InternedProject {
            group: InternedGroup::try_parse(&project.group, interner)?,
            artifact: InternedArtifactName::try_parse(&project.artifact, interner)?,
            version: InternedVersion::try_parse(&project.version, interner)?,
            description: project
                .description
                .as_ref()
                .map(|d| interner.get_or_intern(d)),
            authors: project
                .author
                .as_ref()
                .map(|authors| authors.iter().map(|a| a.intern(interner)).collect()),
            license: project.license.as_ref().map(|l| interner.get_or_intern(l)),
            builds: project.build.clone(),
            rules: project.rule.clone(),
            toolchains: project.toolchain.clone(),
            subprojects: project.subproject.clone(),
            dependencies: project.dependency.as_ref().map(|deps| {
                deps.iter()
                    .map(|(k, v)| (interner.get_or_intern(k), v.clone()))
                    .collect()
            }),
            mount_configuration: project
                .mount_config
                .as_ref()
                .map(|m| interner.get_or_intern(m)),
            configurations: project.config.as_ref().map(|cfg| {
                cfg.iter()
                    .map(|(k, v)| (interner.get_or_intern(k), v.clone()))
                    .collect()
            }),
        })
    }

    pub fn into_raw(&self, interner: &crate::id::Interner) -> Project {
        Project {
            group: interner.resolve(&self.group.0).to_string(),
            artifact: interner.resolve(&self.artifact.0).to_string(),
            version: interner.resolve(&self.version.0).to_string(),
            description: self
                .description
                .as_ref()
                .map(|d| interner.resolve(d).to_string()),
            author: self.authors.as_ref().map(|authors| {
                authors
                    .iter()
                    .map(|a| {
                        let s = interner.resolve(&a.0);
                        // 解析回 Author 结构
                        s.parse().unwrap_or(Author {
                            name: s.to_string(),
                            email: "".to_string(),
                        })
                    })
                    .collect()
            }),
            license: self
                .license
                .as_ref()
                .map(|l| interner.resolve(l).to_string()),
            build: self.builds.clone(),
            rule: self.rules.clone(),
            toolchain: self.toolchains.clone(),
            subproject: self.subprojects.clone(),
            dependency: self.dependencies.as_ref().map(|deps| {
                deps.iter()
                    .map(|(k, v)| (interner.resolve(k).to_string(), v.clone()))
                    .collect()
            }),
            mount_config: self
                .mount_configuration
                .as_ref()
                .map(|m| interner.resolve(m).to_string()),
            config: self.configurations.as_ref().map(|cfg| {
                cfg.iter()
                    .map(|(k, v)| (interner.resolve(k).to_string(), v.clone()))
                    .collect()
            }),
        }
    }
}

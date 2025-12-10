use std::path::PathBuf;

use crate::{Pattern, author::Author, id::PackageId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS, Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[ts(export, export_to = "project.d.ts")]
#[ts(optional_fields)]
pub struct Project {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Option<Vec<Author>>,
    pub license: Option<String>,
    pub builds: Option<Pattern>,
    pub rules: Option<Pattern>,
    pub toolchains: Option<Pattern>,
    pub subprojects: Option<Pattern>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResolvedProject {
    pub package_id: PackageId,
    pub description: Option<String>,
    pub authors: Option<Vec<Author>>,
    pub license: Option<String>,
    pub builds: Vec<PathBuf>,
    pub rules: Vec<PathBuf>,
    pub toolchains: Vec<PathBuf>,
    pub subprojects: Vec<PathBuf>,
}

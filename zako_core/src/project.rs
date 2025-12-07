use crate::{Pattern, access_control::Visibility, author::Author};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

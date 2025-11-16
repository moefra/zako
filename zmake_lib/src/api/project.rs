use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::api::access_control::Visibility;

#[derive(Serialize, Deserialize,Hash, Debug,Eq, PartialEq)]
pub enum ScriptType{
    Project,
    Rule,
    Script
}

#[derive(Serialize, Deserialize,Hash, Debug,Eq, PartialEq)]
pub struct BuildFile{
    files:Vec<String>,
    visibility: Visibility
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectExported {
    pub includes:Vec<BuildFile>,
    pub scripts:HashMap<ScriptType, BuildFile>,
}

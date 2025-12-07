use crate::id::{Id, ResolvedId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigurationValue {
    Boolean(bool),
    Number(i64),
    String(String),
    Identifier(ResolvedId),
    Strings(Vec<String>),
    Identifiers(Vec<ResolvedId>),
}

pub type SimpleConfiguration =
    ::std::collections::HashMap<ResolvedId, ConfigurationValue, ::ahash::RandomState>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Configuration {
    parent: Option<Box<Configuration>>,
    this: SimpleConfiguration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    input: Vec<String>,
    output: Vec<String>,
    configuration: SimpleConfiguration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfiguredId {
    id: ResolvedId,
    configuration: Configuration,
}

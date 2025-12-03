use crate::id::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigurationValue {
    Boolean(bool),
    Number(i64),
    String(String),
    Identifier(Id),
    Strings(Vec<String>),
    Identifiers(Vec<Id>),
}

pub type SimpleConfiguration =
    ::std::collections::HashMap<Id, ConfigurationValue, ::ahash::RandomState>;

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
    id: Id,
    configuration: Configuration,
}

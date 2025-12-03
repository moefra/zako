use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
#[serde(rename_all(serialize = "lowercase", deserialize = "lowercase"))]
#[serde(untagged)]
pub enum Visibility {
    Restricted(Vec<String>),
    Private,
    Public,
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransitiveLevel {
    Interface,
    Public,
    Private,
}

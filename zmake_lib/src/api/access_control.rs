use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug,Hash,Eq, PartialEq)]
#[serde(tag = "t", content = "c")]
pub enum Visibility{
    VisibleToArtifact(Vec<String>),
    VisibleToFile(Vec<String>),
    VisibleToDir(Vec<String>),
    Private,
    Public
}

#[derive(Serialize, Deserialize, Debug,Hash,Eq, PartialEq)]
#[serde(tag = "t", content = "c")]
pub enum TransitiveLevel{
    Interface,
    Public,
    Private
}

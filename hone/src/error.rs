use std::{rc::Rc, sync::Arc};

use crate::node::NodeKey;

#[derive(thiserror::Error, Debug)]
pub enum HoneError {
    #[error("Cycle detected when search `{current:?}` called from `{caller:?}`")]
    CycleDetected {
        caller: Vec<String>,
        current: String,
    },
    #[error("Missing dependency `{missing:?}` when search `{caller:?}`")]
    MissingDependency { caller: String, missing: String },
    #[error("Other error: {0}")]
    Other(#[from] eyre::Report),
    #[error("Unknown(may a bug) error: {0}")]
    UnexpectedError(String),
    #[error("Aggregative error:{0:?}")]
    AggregativeError(Vec<Arc<HoneError>>),
    #[error("Invalid database state: {0}")]
    InvalidDatabaseState(String),
}

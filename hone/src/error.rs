use std::sync::Arc;


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
    #[error("Canceled: {reason:?}")]
    Canceled {
        reason: Option<zako_cancel::CancelReason>,
    },
    #[error("get IO error `{0}` when access `{1}`")]
    IOError(#[source] std::io::Error, String),
}

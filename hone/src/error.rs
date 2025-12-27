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
    #[error("Assertion: {1} <=> `{0}` failed, this should be seems as a bug of program")]
    AssertionFailed(String, String),
    #[error("Unexpected error: {0}d, this should be seems as a bug of hone")]
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
    #[error("From Arc<HoneError> error: {0:?}")]
    SharedError(#[from] Arc<HoneError>),
}

#[macro_export]
macro_rules! assert {
    ($message:expr, $condition:expr) => {
        if !$condition {
            return Err(HoneError::AssertionFailed(
                $message.to_string(),
                format!("{}", ::std::stringify!($condition)),
            ));
        }
    };
}

#[macro_export]
macro_rules! debug_assert {
    ($message:expr, $condition:expr) => {
        #[cfg(debug_assertions)]
        if !$condition {
            return Err(HoneError::AssertionFailed(
                $message.to_string(),
                format!("{}", ::std::stringify!($condition)),
            ));
        }
    };
}

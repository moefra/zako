use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pattern {
    Includes(Vec<String>),
    IncludesAndExcludes {
        includes: Option<Vec<String>>,
        excludes: Option<Vec<String>>,
    },
}

impl Pattern {
    pub fn new(includes: Vec<String>, excludes: Vec<String>) -> Self {
        Pattern::IncludesAndExcludes {
            includes: Some(includes),
            excludes: Some(excludes),
        }
    }
}

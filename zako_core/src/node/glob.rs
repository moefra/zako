use crate::{
    intern::InternedAbsolutePath,
    path::interned::InternedNeutralPath,
    pattern::{InternedPattern, PatternGroup},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub enum GlobRequest {
    Single(InternedPattern),
    Multiple(PatternGroup),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct Glob {
    pub base_path: InternedAbsolutePath,
    pub request: GlobRequest,
}

#[derive(Debug, Clone, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct GlobResult {
    pub paths: Vec<InternedAbsolutePath>,
}

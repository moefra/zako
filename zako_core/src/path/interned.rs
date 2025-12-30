use crate::intern::InternedString;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Copy,
    rkyv::Deserialize,
    rkyv::Serialize,
    rkyv::Archive,
)]
pub struct InternedNeutralPath {
    interned: InternedString,
}

impl InternedNeutralPath {
    /// Caller should ensure that `interned` is a valid NeutralPath string
    pub unsafe fn from_raw(interned: InternedString) -> Self {
        Self { interned }
    }
}

impl AsRef<InternedString> for InternedNeutralPath {
    fn as_ref(&self) -> &InternedString {
        &self.interned
    }
}

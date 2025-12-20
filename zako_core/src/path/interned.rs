use crate::intern::InternedString;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub struct InternedNeutralPath {
    interned: InternedString,
}

impl InternedNeutralPath {
    /// Caller should ensure that `interned` is a valid NeutralPath string
    pub unsafe fn from_raw(interned: InternedString) -> Self {
        Self { interned }
    }

    pub fn interned(&self) -> &InternedString {
        &self.interned
    }
}

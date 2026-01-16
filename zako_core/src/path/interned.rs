use crate::{
    intern::{InternedString, Interner, Resolvable, Uninternable},
    path::NeutralPath,
};

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Copy, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive,
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

impl Uninternable for InternedNeutralPath {
    type Uninterned = NeutralPath;

    fn unintern(&self, interner: &Interner) -> eyre::Result<Self::Uninterned> {
        Ok(unsafe { NeutralPath::from_unchecked(interner.resolve(self.interned)?) })
    }
}

impl Resolvable for InternedNeutralPath {
    fn resolve<'a>(&self, interner: &'a Interner) -> eyre::Result<&'a str> {
        Ok(interner.resolve(self.interned)?)
    }
}

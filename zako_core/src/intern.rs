pub type InternedString = ::zako_interner::U32NonZeroKey;

pub type Interner = ::zako_interner::ThreadedInterner;

/// A path that has been interned.
///
/// It must be absolute path.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    rkyv::Deserialize,
    rkyv::Serialize,
    rkyv::Archive,
)]
pub struct InternedAbsolutePath {
    pub interned: InternedString,
}

impl InternedAbsolutePath {
    pub fn new(
        path: &str,
        interner: &mut Interner,
    ) -> Result<Option<Self>, ::zako_interner::InternerError> {
        // check if the interned string is an absolute path
        {
            let path = std::path::Path::new(path);
            if !path.is_absolute() {
                return Ok(None);
            }
        }

        Ok(Some(Self {
            interned: interner.get_or_intern(path)?,
        }))
    }
    pub fn from_interned(
        interned: InternedString,
        interner: &Interner,
    ) -> Result<Option<Self>, ::zako_interner::InternerError> {
        let s = interner.resolve(&interned)?;
        let path = std::path::Path::new(s);
        if !path.is_absolute() {
            return Ok(None);
        }

        Ok(Some(Self { interned }))
    }

    pub(crate) unsafe fn _from_interned_unchecked(interned: InternedString) -> Self {
        Self { interned }
    }
}

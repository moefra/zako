use std::num::NonZeroU32;

use ::zako_interner::Key;

use crate::context::BuildContext;

pub type InternedString = ::zako_interner::U32NonZeroKey;

pub type Interner = ::zako_interner::PersistentInterner<InternedString>;

/// A path that has been interned.
///
/// It must be absolute path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternedAbsolutePath {
    pub interned: InternedString,
}

impl InternedAbsolutePath {
    pub fn new(path: &str, interner: &mut Interner) -> Option<Self> {
        // check if the interned string is an absolute path
        {
            let path = std::path::Path::new(path);
            if !path.is_absolute() {
                return None;
            }
        }

        Some(Self {
            interned: interner.get_or_intern(path),
        })
    }
    pub fn from_interned(interned: InternedString, interner: &Interner) -> Option<Self> {
        let s = interner.resolve(&interned);
        let path = std::path::Path::new(&s);
        if !path.is_absolute() {
            return None;
        }

        Some(Self { interned })
    }

    pub(crate) unsafe fn from_interned_unchecked(interned: InternedString) -> Self {
        Self { interned }
    }
}

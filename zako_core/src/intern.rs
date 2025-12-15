use std::num::NonZeroU32;

use lasso::Key;

use crate::context::BuildContext;

/// Why NonZeroU32?
///
/// It can make `Option<InternedString>` take only 4 bytes instead of 8 bytes,
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternedString(NonZeroU32);

unsafe impl Key for InternedString {
    #[inline]
    fn into_usize(self) -> usize {
        self.0.get() as usize - 1
    }
    /// Returns `None` if `int` is greater than `u32::MAX - 1`
    #[inline]
    fn try_from_usize(int: usize) -> Option<Self> {
        if int < u32::MAX as usize {
            // Safety: The integer is less than the max value and then incremented by one, meaning that
            // is is impossible for a zero to inhabit the NonZeroU32
            unsafe { Some(Self(NonZeroU32::new_unchecked(int as u32 + 1))) }
        } else {
            None
        }
    }
}

impl InternedString {
    pub fn as_u64(&self) -> u64 {
        self.0.get() as u64
    }
}

pub type Interner = ::lasso::ThreadedRodeo<InternedString>;

/// A path that has been interned.
///
/// It must be absolute path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternedAbsolutePath {
    interned: InternedString,
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
        let path = std::path::Path::new(s);
        if !path.is_absolute() {
            return None;
        }

        Some(Self { interned })
    }

    pub(crate) unsafe fn from_interned_unchecked(interned: InternedString) -> Self {
        Self { interned }
    }

    pub fn interned(&self) -> &InternedString {
        &self.interned
    }

    pub fn as_u64(&self) -> u64 {
        self.interned.as_u64()
    }
}

pub trait Internable<C> {
    type Interned;

    fn intern(self, context: &C) -> Result<Self::Interned, String>;
    fn resolve(interned: &Self::Interned, context: &C) -> Self;
}

impl Internable<BuildContext> for String {
    type Interned = InternedString;

    fn intern(self, context: &BuildContext) -> Result<InternedString, String> {
        Ok(context.interner().get_or_intern(self.as_str()))
    }

    fn resolve(interned: &InternedString, context: &BuildContext) -> String {
        context.interner().resolve(interned).to_string()
    }
}

impl Internable<BuildContext> for Vec<String> {
    type Interned = Vec<InternedString>;

    fn intern(self, context: &BuildContext) -> Result<Vec<InternedString>, String> {
        Ok(self
            .into_iter()
            .map(|s| context.interner().get_or_intern(&s))
            .collect())
    }

    fn resolve(interned: &Vec<InternedString>, context: &BuildContext) -> Vec<String> {
        interned
            .iter()
            .map(|s| context.interner().resolve(s).to_string())
            .collect()
    }
}

impl Internable<BuildContext> for Option<String> {
    type Interned = Option<InternedString>;

    fn intern(self, context: &BuildContext) -> Result<Option<InternedString>, String> {
        Ok(self.map(|s| context.interner().get_or_intern(&s)))
    }

    fn resolve(interned: &Option<InternedString>, context: &BuildContext) -> Option<String> {
        interned.map(|s| context.interner().resolve(&s).to_string())
    }
}

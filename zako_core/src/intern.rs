use eyre::Context;
use smol_str::SmolStr;

pub type InternedString = ::zako_interner::U32NonZeroKey;

pub type Interner = ::zako_interner::ThreadedInterner;

pub trait Resolvable<C = Interner> {
    #[must_use]
    fn resolve<'a>(&self, interner: &'a C) -> eyre::Result<&'a str>;
}

pub trait Internable<C = Interner> {
    type Interned;

    #[must_use]
    fn intern(self, interner: &C) -> eyre::Result<Self::Interned>;
}

pub trait Uninternable<C = Interner> {
    type Uninterned;

    #[must_use]
    fn unintern(&self, interner: &C) -> eyre::Result<Self::Uninterned>;
}

impl Internable for SmolStr {
    type Interned = InternedString;

    fn intern(self, interner: &Interner) -> eyre::Result<Self::Interned> {
        interner
            .get_or_intern(self.as_str())
            .wrap_err("failed to intern")
    }
}

impl Uninternable for InternedString {
    type Uninterned = SmolStr;

    fn unintern(&self, interner: &Interner) -> eyre::Result<Self::Uninterned> {
        interner
            .resolve(self)
            .map(|s| SmolStr::new(s))
            .wrap_err("failed to unintern")
    }
}

impl Resolvable for InternedString {
    fn resolve<'i>(&self, interner: &'i Interner) -> eyre::Result<&'i str> {
        interner.resolve(self).wrap_err("failed to resolve")
    }
}

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
    interned: InternedString,
}

impl AsRef<InternedString> for InternedAbsolutePath {
    fn as_ref(&self) -> &InternedString {
        &self.interned
    }
}

impl Resolvable for InternedAbsolutePath {
    fn resolve<'a>(&self, interner: &'a Interner) -> eyre::Result<&'a str> {
        interner
            .as_ref()
            .resolve(self)
            .wrap_err("failed to resolve")
    }
}

impl InternedAbsolutePath {
    pub fn new(
        path: &str,
        interner: &Interner,
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

impl<T, C> Internable<C> for Vec<T>
where
    T: Internable<C>,
{
    type Interned = Vec<T::Interned>;

    fn intern(self, interner: &C) -> eyre::Result<Self::Interned> {
        let mut result = Vec::with_capacity(self.len());

        for item in self.into_iter() {
            result.push(item.intern(interner)?);
        }

        Ok(result)
    }
}

impl<T, C> Internable<C> for Option<T>
where
    T: Internable<C>,
{
    type Interned = Option<T::Interned>;

    fn intern(self, interner: &C) -> eyre::Result<Self::Interned> {
        match self {
            Some(item) => Ok(Some(item.intern(interner)?)),
            None => Ok(None),
        }
    }
}

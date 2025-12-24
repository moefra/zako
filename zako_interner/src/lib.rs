use std::num::{NonZeroU32, NonZeroUsize};

use lasso::{Capacity, Key, Reader, ThreadedRodeo};
use rkyv::{
    Archive, Archived, Deserialize, Place, Resolver, Serialize,
    rancor::Fallible,
    vec::{ArchivedVec, VecResolver},
    with::{ArchiveWith, DeserializeWith, SerializeWith},
};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[repr(transparent)]
pub struct U32NonZeroKey(NonZeroU32);

impl U32NonZeroKey {
    pub fn as_u64(&self) -> u64 {
        self.0.get() as u64
    }
}

unsafe impl Key for U32NonZeroKey {
    #[inline]
    fn into_usize(self) -> usize {
        self.0.get() as usize - 1
    }
    #[inline]
    fn try_from_usize(int: usize) -> Option<Self> {
        if int < u32::MAX as usize {
            unsafe { Some(Self(NonZeroU32::new_unchecked(int as u32 + 1))) }
        } else {
            None
        }
    }
}

impl AsRef<U32NonZeroKey> for U32NonZeroKey {
    fn as_ref(&self) -> &U32NonZeroKey {
        self
    }
}

pub type LassoInterner = ThreadedRodeo<U32NonZeroKey, ::ahash::RandomState>;

#[derive(serde::Serialize, serde::Deserialize, Archive, Serialize, Deserialize)]
pub struct ThreadedInterner {
    #[rkyv(with=ArchivedThreadedRodeoToVec)]
    interner: LassoInterner,
}

struct ArchivedThreadedRodeoToVec;

impl ArchiveWith<LassoInterner> for ArchivedThreadedRodeoToVec {
    type Archived = Archived<Vec<u8>>;
    type Resolver = Resolver<Vec<u8>>;

    fn resolve_with(field: &LassoInterner, resolver: VecResolver, out: Place<ArchivedVec<u8>>) {
        ArchivedVec::resolve_from_slice(&postcard::to_allocvec(field).unwrap(), resolver, out);
    }
}

impl<S: Fallible + ?Sized + rkyv::ser::Allocator + rkyv::ser::Writer>
    SerializeWith<LassoInterner, S> for ArchivedThreadedRodeoToVec
where
    Vec<u8>: Serialize<S>,
{
    fn serialize_with(
        field: &LassoInterner,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedVec::serialize_from_slice(&postcard::to_allocvec(field).unwrap(), serializer)
    }
}

impl<D: Fallible + ?Sized> DeserializeWith<Archived<Vec<u8>>, LassoInterner, D>
    for ArchivedThreadedRodeoToVec
where
    Archived<Vec<u8>>: Deserialize<Vec<u8>, D>,
{
    fn deserialize_with(
        field: &Archived<Vec<u8>>,
        deserializer: &mut D,
    ) -> Result<LassoInterner, D::Error> {
        Ok(postcard::from_bytes(field).unwrap())
    }
}

impl ThreadedInterner {
    #[inline]
    pub fn interner(&self) -> &ThreadedRodeo<U32NonZeroKey, ::ahash::RandomState> {
        &self.interner
    }

    pub fn new() -> Self {
        Self {
            interner: ThreadedRodeo::with_capacity_and_hasher(
                Capacity::new(1024, NonZeroUsize::new(1024 * 8).unwrap()),
                ::ahash::RandomState::default(),
            ),
        }
    }

    #[inline]
    pub fn resolve(&self, key: impl AsRef<U32NonZeroKey>) -> &str {
        self.interner.resolve(key.as_ref())
    }

    #[inline]
    pub fn get_or_intern(&self, val: impl AsRef<str>) -> U32NonZeroKey {
        self.interner.get_or_intern(val.as_ref())
    }

    #[inline]
    pub fn get_or_intern_static(&self, val: &'static str) -> U32NonZeroKey {
        self.interner.get_or_intern_static(val)
    }
}

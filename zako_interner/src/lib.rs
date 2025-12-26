use std::num::{NonZeroU32, NonZeroUsize};

use lasso::{Capacity, Key, ThreadedRodeo};
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Archive, Serialize, Deserialize)]
pub struct ThreadedInterner {
    #[rkyv(with=ArchivedThreadedRodeoToVec)]
    interner: LassoInterner,
    id: u64,
}

struct ArchivedThreadedRodeoToVec;

impl ArchiveWith<LassoInterner> for ArchivedThreadedRodeoToVec {
    type Archived = Archived<Vec<u8>>;
    type Resolver = Resolver<Vec<u8>>;

    #[allow(clippy::expect_used)]
    fn resolve_with(field: &LassoInterner, resolver: VecResolver, out: Place<ArchivedVec<u8>>) {
        ArchivedVec::resolve_from_slice(
            &postcard::to_allocvec(field).expect(
                "Failed to convert ::zako_interner::ThreadedInterner::LassoInterner to Vec<u8> for ArchiveWith",
            ),
            resolver,
            out,
        );
    }
}

impl<S: Fallible + ?Sized + rkyv::ser::Allocator + rkyv::ser::Writer>
    SerializeWith<LassoInterner, S> for ArchivedThreadedRodeoToVec
where
    Vec<u8>: Serialize<S>,
{
    #[allow(clippy::expect_used)]
    fn serialize_with(
        field: &LassoInterner,
        serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        ArchivedVec::serialize_from_slice(
            &postcard::to_allocvec(field).expect(
                "Failed to convert ::zako_interner::ThreadedInterner::LassoInterner to Vec<u8> for SerializeWith",
            ),
            serializer,
        )
    }
}

impl<D: Fallible + ?Sized> DeserializeWith<Archived<Vec<u8>>, LassoInterner, D>
    for ArchivedThreadedRodeoToVec
where
    Archived<Vec<u8>>: Deserialize<Vec<u8>, D>,
{
    #[allow(clippy::expect_used)]
    fn deserialize_with(
        field: &Archived<Vec<u8>>,
        _deserializer: &mut D,
    ) -> Result<LassoInterner, D::Error> {
        Ok(postcard::from_bytes(field).expect(
            "Failed to convert Vec<u8> to ::zako_interner::ThreadedInterner::LassoInterner for DeserializeWith",
        ))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, thiserror::Error)]
pub enum InternerError {
    #[error("Failed to create ThreadedInterner: {0}")]
    CreationError(String),
    #[error("Key out of bounds(Interner ID: {0}): {1:?}")]
    KeyOutOfBounds(u64, U32NonZeroKey),
    #[error("Get a lasso error(Interner ID: {0}): {1}")]
    LassoError(u64, #[source] lasso::LassoError),
}

impl ThreadedInterner {
    #[must_use]
    #[inline]
    pub fn interner(&self) -> &ThreadedRodeo<U32NonZeroKey, ::ahash::RandomState> {
        &self.interner
    }

    #[must_use]
    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }

    #[inline]
    #[must_use]
    pub fn new() -> Result<Self, InternerError> {
        Ok(Self {
            id: ::getrandom::u64()
                .map_err(|err| InternerError::CreationError(format!("{:?}", err)))?,
            interner: ThreadedRodeo::with_capacity_and_hasher(
                Capacity::new(
                    1024,
                    NonZeroUsize::new(1024 * 8).ok_or(InternerError::CreationError(
                        "Failed to create NonZeroUsize for capacity of ThreadedInterner".into(),
                    ))?,
                ),
                ::ahash::RandomState::default(),
            ),
        })
    }

    #[inline]
    #[must_use]
    pub fn resolve(&self, key: impl AsRef<U32NonZeroKey>) -> Result<&str, InternerError> {
        self.interner
            .try_resolve(key.as_ref())
            .ok_or_else(|| InternerError::KeyOutOfBounds(self.id, key.as_ref().clone()))
    }

    #[inline]
    #[must_use]
    pub fn get_or_intern(&self, val: impl AsRef<str>) -> Result<U32NonZeroKey, InternerError> {
        self.interner
            .try_get_or_intern(val.as_ref())
            .map_err(|err| InternerError::LassoError(self.id, err))
    }

    #[inline]
    #[must_use]
    pub fn get_or_intern_static(&self, val: &'static str) -> Result<U32NonZeroKey, InternerError> {
        self.interner
            .try_get_or_intern_static(val)
            .map_err(|err| InternerError::LassoError(self.id, err))
    }
}

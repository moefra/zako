use std::{hash::Hash, num::NonZeroU32};

/// A trait for types that can be used as keys in the interner.
///
/// This is similar to lasso's `Key` trait.
pub trait Key: Copy + Eq + Hash + Send + Sync + 'static {
    /// Returns the [u64] that represents the current key
    fn into_u64(self) -> u64;

    /// Attempts to create a key from a [u64], returning `None` if it fails
    fn try_from_u64(int: u64) -> Option<Self>;
}

/// Why NonZeroU32?
///
/// It can make `Option<InternedString>` take only 4 bytes instead of 8 bytes,
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    ::serde::Serialize,
    ::serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct U32NonZeroKey(NonZeroU32);

impl Key for U32NonZeroKey {
    #[inline]
    fn into_u64(self) -> u64 {
        self.0.get() as u64 - 1
    }

    /// Returns `None` if `int` is greater than `u32::MAX - 1`
    #[inline]
    fn try_from_u64(int: u64) -> Option<Self> {
        if int < u32::MAX as u64 {
            // Safety: The integer is less than the max value and then incremented by one, meaning that
            // is is impossible for a zero to inhabit the NonZeroU32
            unsafe { Some(Self(NonZeroU32::new_unchecked(int as u32 + 1))) }
        } else {
            None
        }
    }
}

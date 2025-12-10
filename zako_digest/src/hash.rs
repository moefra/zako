use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use xxhash_rust::xxh3;

/// Compute the XXHash3 128-bit hash of the object.
///
/// It should be platform independent and consistent across different runs.
///
/// isize and usize are converted to u64 for hashing to ensure cross-platform consistency.
pub trait XXHash3 {
    fn xxhash3_128(&self) -> u128;
}

/// Allow calling on &T
impl<T: XXHash3 + ?Sized> XXHash3 for &T {
    fn xxhash3_128(&self) -> u128 {
        (**self).xxhash3_128()
    }
}

/// Allow calling on &mut T
impl<T: XXHash3 + ?Sized> XXHash3 for &mut T {
    fn xxhash3_128(&self) -> u128 {
        (**self).xxhash3_128()
    }
}

/// Allow calling on Box<T>
impl XXHash3 for Vec<u8> {
    fn xxhash3_128(&self) -> u128 {
        xxh3::xxh3_128(self.as_slice())
    }
}
/// Allow calling on String
impl XXHash3 for String {
    fn xxhash3_128(&self) -> u128 {
        xxh3::xxh3_128(self.as_bytes())
    }
}
/// Allow calling on &str
impl XXHash3 for str {
    fn xxhash3_128(&self) -> u128 {
        xxh3::xxh3_128(self.as_bytes())
    }
}
/// Allow calling on PathBuf
impl XXHash3 for PathBuf {
    fn xxhash3_128(&self) -> u128 {
        xxh3::xxh3_128(self.as_os_str().as_bytes())
    }
}
/// Allow calling on Path
impl XXHash3 for Path {
    fn xxhash3_128(&self) -> u128 {
        xxh3::xxh3_128(self.as_os_str().as_bytes())
    }
}
/// Allow calling on OsString
impl XXHash3 for OsString {
    fn xxhash3_128(&self) -> u128 {
        xxh3::xxh3_128(self.as_bytes())
    }
}
/// Allow calling on OsStr
impl XXHash3 for OsStr {
    fn xxhash3_128(&self) -> u128 {
        xxh3::xxh3_128(self.as_bytes())
    }
}
/// Allow calling on Rc<T>
impl<T: XXHash3 + ?Sized> XXHash3 for Rc<T> {
    fn xxhash3_128(&self) -> u128 {
        (**self).xxhash3_128()
    }
}
/// Allow calling on Arc<T>
impl<T: XXHash3 + ?Sized> XXHash3 for Arc<T> {
    fn xxhash3_128(&self) -> u128 {
        (**self).xxhash3_128()
    }
}
/// Allow calling on Box<T>
impl<T: XXHash3 + ?Sized> XXHash3 for Box<T> {
    fn xxhash3_128(&self) -> u128 {
        (**self).xxhash3_128()
    }
}
/// Allow calling on Option<T>
impl<T: XXHash3 + Sized> XXHash3 for Option<T> {
    fn xxhash3_128(&self) -> u128 {
        // Using streaming update is slightly more efficient and more standard than generating intermediate Hash and then hashing again.
        let mut hasher = xxh3::Xxh3::new();
        match self {
            Some(value) => {
                hasher.update(&[1u8]); // Tag
                let h = value.xxhash3_128(); // value
                hasher.update(&h.to_le_bytes());
            }
            None => {
                hasher.update(&[0u8]); // Tag
                hasher.update(&UNIT_HASH_MAGIC.to_le_bytes()); // ()
            }
        };
        hasher.digest128()
    }
}

pub const UNIT_HASH_MAGIC: u128 = 0x0011_2233_4455_6677_8899_AABB_CCDD_EEFF;

impl XXHash3 for () {
    fn xxhash3_128(&self) -> u128 {
        UNIT_HASH_MAGIC
    }
}

macro_rules! impl_xxhash3_for_fixed_numbers {
    ($($t:ty),*) => {
        $(
            impl XXHash3 for $t {
                fn xxhash3_128(&self) -> u128 {
                    xxh3::xxh3_128(&self.to_le_bytes())
                }
            }
        )*
    }
}

impl_xxhash3_for_fixed_numbers!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

macro_rules! impl_xxhash3_for_arch_numbers {
    ($($t:ty),*) => {
        $(
            impl XXHash3 for $t {
                fn xxhash3_128(&self) -> u128 {
                    (*self as u64).xxhash3_128()
                }
            }
        )*
    }
}

impl_xxhash3_for_arch_numbers!(usize, isize);

impl XXHash3 for bool {
    fn xxhash3_128(&self) -> u128 {
        let val = if *self { 1u8 } else { 0u8 };
        xxh3::xxh3_128(&[val])
    }
}

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
/// So when use it with primitives like isize/usize, we convert them to fixed size u64 first.
///
/// And we use to_le_bytes to hash primitive numbers like u32 or f64.
pub trait XXHash3 {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3);
    fn xxhash3_128(&self) -> u128 {
        let mut hasher = xxh3::Xxh3::new();
        self.hash_into(&mut hasher);
        hasher.digest128()
    }
}

/// Allow calling on &T
impl<T: XXHash3 + ?Sized> XXHash3 for &T {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        (**self).hash_into(hasher)
    }
}

/// Allow calling on &mut T
impl<T: XXHash3 + ?Sized> XXHash3 for &mut T {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        (**self).hash_into(hasher)
    }
}

/// Allow calling on Box<T>
impl XXHash3 for Vec<u8> {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        hasher.update(self.as_slice());
    }
}
/// Allow calling on String
impl XXHash3 for String {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        hasher.update(self.as_bytes());
    }
}
/// Allow calling on &str
impl XXHash3 for str {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        hasher.update(self.as_bytes());
    }
}
/// Allow calling on PathBuf
impl XXHash3 for PathBuf {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        hasher.update(self.as_os_str().as_bytes());
    }
}
/// Allow calling on Path
impl XXHash3 for Path {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        hasher.update(self.as_os_str().as_bytes());
    }
}
/// Allow calling on OsString
impl XXHash3 for OsString {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        hasher.update(self.as_bytes());
    }
}
/// Allow calling on OsStr
impl XXHash3 for OsStr {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        hasher.update(self.as_bytes());
    }
}
/// Allow calling on Rc<T>
impl<T: XXHash3 + ?Sized> XXHash3 for Rc<T> {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        (**self).hash_into(hasher)
    }
}
/// Allow calling on Arc<T>
impl<T: XXHash3 + ?Sized> XXHash3 for Arc<T> {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        (**self).hash_into(hasher)
    }
}
/// Allow calling on Box<T>
impl<T: XXHash3 + ?Sized> XXHash3 for Box<T> {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        (**self).hash_into(hasher)
    }
}
/// Allow calling on Option<T>
///
/// If options it none, it will hash a tag byte 0u8 and the unit hash
/// If option is some, it will hash a tag byte 1u8 and the value's hash.
impl<T: XXHash3 + Sized> XXHash3 for Option<T> {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        // Using streaming update is slightly more efficient and more standard than generating intermediate Hash and then hashing again.
        match self {
            Some(value) => {
                hasher.update(&[1u8]); // Tag
                value.hash_into(hasher); // value
            }
            None => {
                hasher.update(&[0u8]); // Tag
                ().hash_into(hasher); // use unit as no value
            }
        };
    }
}

pub const UNIT_HASH_MAGIC: u128 = 0x0011_2233_4455_6677_8899_AABB_CCDD_EEFF;

impl XXHash3 for () {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        hasher.update(&UNIT_HASH_MAGIC.to_le_bytes());
    }
}

/// Allow calling on fixed-size numbers
///
/// See [the trait doc](XXHash3) for explanation.
macro_rules! impl_xxhash3_for_fixed_numbers {
    ($($t:ty),*) => {
        $(
            impl XXHash3 for $t {
                fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
                    hasher.update(&self.to_le_bytes());
                }
            }
        )*
    }
}

impl_xxhash3_for_fixed_numbers!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

/// Allow calling on isize and usize
///
/// See [the trait doc](XXHash3) for explanation.
macro_rules! impl_xxhash3_for_arch_numbers {
    ($($t:ty),*) => {
        $(
            impl XXHash3 for $t {
                fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
                    (*self as u64).hash_into(hasher)
                }
            }
        )*
    }
}

impl_xxhash3_for_arch_numbers!(usize, isize);

impl XXHash3 for bool {
    fn hash_into(&self, hasher: &mut xxh3::Xxh3) {
        let val = if *self { 1u8 } else { 0u8 };
        hasher.update(&[val]);
    }
}

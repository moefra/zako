use std::{
    array::TryFromSliceError,
    collections::HashMap,
    ffi::{OsStr, OsString},
    ops::Deref,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use rkyv::{Archive, Deserialize, Serialize};

/// The trait means a object can be hashed into a blake3 hash.
pub trait Blake3Hash {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher);
    fn get_blake3(&self) -> blake3::Hash {
        let mut hasher = blake3::Hasher::new();
        self.hash_into_blake3(&mut hasher);
        hasher.finalize()
    }
}

macro_rules! impl_blake3_for_tuple {
    ($($ty:ident),*) => {
        impl<$($ty: Blake3Hash),*> Blake3Hash for ($($ty,)*) {
            fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
                // 使用模式匹配解构元组
                #[allow(non_snake_case)]
                let ($($ty,)*) = &self;
                // 对每个元素调用 hash()
                $($ty.hash_into_blake3(hasher);)*
            }
        }
    };
}

impl_blake3_for_tuple!(A);
impl_blake3_for_tuple!(A, B);
impl_blake3_for_tuple!(A, B, C);
impl_blake3_for_tuple!(A, B, C, D);
impl_blake3_for_tuple!(A, B, C, D, E);
impl_blake3_for_tuple!(A, B, C, D, E, F);
impl_blake3_for_tuple!(A, B, C, D, E, F, G);
impl_blake3_for_tuple!(A, B, C, D, E, F, G, H);
impl_blake3_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_blake3_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_blake3_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_blake3_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

/// Allow calling on &T
impl<T: Blake3Hash + ?Sized> Blake3Hash for &T {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        (**self).hash_into_blake3(hasher)
    }
}

/// Allow calling on &mut T
impl<T: Blake3Hash + ?Sized> Blake3Hash for &mut T {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        (**self).hash_into_blake3(hasher)
    }
}

impl Blake3Hash for smol_str::SmolStr {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(self.as_bytes());
    }
}

/// Allow calling on String
impl Blake3Hash for String {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(self.as_bytes());
    }
}

/// Allow calling on &str
impl Blake3Hash for str {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(self.as_bytes());
    }
}
/// Allow calling on PathBuf
impl Blake3Hash for PathBuf {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(self.as_os_str().as_bytes());
    }
}
/// Allow calling on Path
impl Blake3Hash for Path {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(self.as_os_str().as_bytes());
    }
}
/// Allow calling on OsString
impl Blake3Hash for OsString {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(self.as_bytes());
    }
}
/// Allow calling on OsStr
impl Blake3Hash for OsStr {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(self.as_bytes());
    }
}
/// Allow calling on Rc<T>
impl<T: Blake3Hash + ?Sized> Blake3Hash for Rc<T> {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        (**self).hash_into_blake3(hasher)
    }
}
/// Allow calling on Arc<T>
impl<T: Blake3Hash + ?Sized> Blake3Hash for Arc<T> {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        (**self).hash_into_blake3(hasher)
    }
}
/// Allow calling on Box<T>
impl<T: Blake3Hash + ?Sized> Blake3Hash for Box<T> {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        (**self).hash_into_blake3(hasher)
    }
}

impl<T: Blake3Hash + Sized + Ord + Eq> Blake3Hash for Vec<T> {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.len().hash_into_blake3(hasher);
        let mut values: Vec<&T> = self.iter().collect();
        values.sort();

        for value in values.into_iter() {
            value.hash_into_blake3(hasher);
        }
    }
}

impl<T: Blake3Hash + Sized + Ord + std::hash::Hash + Eq, V: Blake3Hash, S> Blake3Hash
    for HashMap<T, V, S>
{
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.len().hash_into_blake3(hasher);

        let mut pairs = self.iter().collect::<Vec<_>>();
        pairs.sort_by_key(|(k, _)| *k);

        for pair in pairs.into_iter() {
            pair.0.hash_into_blake3(hasher);
            pair.1.hash_into_blake3(hasher);
        }
    }
}

/// Allow calling on Option<T>
///
/// If options it none, it will hash a tag byte 0u8 and the unit hash
/// If option is some, it will hash a tag byte 1u8 and the value's hash.
impl<T: Blake3Hash + Sized> Blake3Hash for Option<T> {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        match self {
            Some(value) => {
                hasher.update(&[1u8]); // Tag
                value.hash_into_blake3(hasher); // value
            }
            None => {
                hasher.update(&[0u8]); // Tag
                ().hash_into_blake3(hasher); // use unit as no value
            }
        };
    }
}

pub const UNIT_HASH_MAGIC: u128 = 0x0011_2233_4455_6677_8899_AABB_CCDD_EEFF;

impl Blake3Hash for () {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(&UNIT_HASH_MAGIC.to_le_bytes());
    }
}

/// Allow calling on fixed-size numbers
///
/// See [the trait doc](XXHash3) for explanation.
macro_rules! impl_blake3_hash_for_fixed_numbers {
    ($($t:ty),*) => {
        $(
            impl Blake3Hash for $t {
                fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
                    hasher.update(&self.to_le_bytes());
                }
            }
        )*
    }
}

impl_blake3_hash_for_fixed_numbers!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

/// Allow calling on isize and usize
///
/// See [the trait doc](XXHash3) for explanation.
macro_rules! impl_blake3_hash_for_arch_numbers {
    ($($t:ty),*) => {
        $(
            impl Blake3Hash for $t {
                fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
                    (*self as u64).hash_into_blake3(hasher)
                }
            }
        )*
    }
}

impl_blake3_hash_for_arch_numbers!(usize, isize);

impl Blake3Hash for bool {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        let val = if *self { 1u8 } else { 0u8 };
        hasher.update(&[val]);
    }
}

#[derive(Clone, Debug, Hash, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct Hash {
    hash_bytes: [u8; 32],
}

impl Hash {
    pub fn new(hash_bytes: [u8; 32]) -> Self {
        Self { hash_bytes }
    }

    pub fn from_bytes(hash_bytes: &[u8; 32]) -> Self {
        Self {
            hash_bytes: *hash_bytes,
        }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.hash_bytes
    }

    pub fn to_hex(&self) -> arrayvec::ArrayString<64> {
        let mut hex: arrayvec::ArrayString<64> = arrayvec::ArrayString::new();
        unsafe {
            hex.set_len(64);
            ::hex::encode_to_slice(self.hash_bytes, hex.as_mut().as_bytes_mut()).unwrap();
        }
        hex
    }
}

impl From<[u8; 32]> for Hash {
    fn from(value: [u8; 32]) -> Self {
        Hash { hash_bytes: value }
    }
}

impl From<&[u8; 32]> for Hash {
    fn from(value: &[u8; 32]) -> Self {
        Hash { hash_bytes: *value }
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = TryFromSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Hash {
            hash_bytes: value.try_into()?,
        })
    }
}

impl Into<[u8; 32]> for Hash {
    fn into(self) -> [u8; 32] {
        self.hash_bytes
    }
}

impl Deref for Hash {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.hash_bytes
    }
}

impl From<Hash> for blake3::Hash {
    fn from(value: Hash) -> Self {
        blake3::Hash::from_bytes(value.hash_bytes)
    }
}

impl From<blake3::Hash> for Hash {
    fn from(value: blake3::Hash) -> Self {
        Hash::from_bytes(value.as_bytes())
    }
}

impl Blake3Hash for Hash {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(self.as_bytes());
    }
}

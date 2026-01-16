use std::{
    array::TryFromSliceError,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    marker::PhantomData,
    rc::Rc,
    sync::Arc,
};

use camino::{Utf8Path, Utf8PathBuf};
use rkyv::{Archive, Deserialize, Serialize};

/// The trait means a object can be hashed into a blake3 hash.
pub trait XXHash3Hash {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3);
    fn get_xxhash3(&self) -> Hash {
        let mut hasher = xxhash_rust::xxh3::Xxh3::new();
        self.hash_into_xxhash3(&mut hasher);
        Hash::new(hasher.digest128())
    }
}

macro_rules! impl_blake3_for_tuple {
    ($($ty:ident),*) => {
        impl<$($ty: XXHash3Hash),*> XXHash3Hash for ($($ty,)*) {
            fn hash_into_xxhash3(&self, hasher: &mut ::xxhash_rust::xxh3::Xxh3) {
                hasher.update(b"::std::tuple::Tuple");
                let len = [0u8; 0].len() $( + { let _ = stringify!($ty); 1 } )*;
                (len as u64).hash_into_xxhash3(hasher);

                // 使用模式匹配解构元组
                #[allow(non_snake_case)]
                let ($($ty,)*) = &self;
                // 对每个元素调用 hash()
                $($ty.hash_into_xxhash3(hasher);)*
            }
        }
    };
}

impl<T> XXHash3Hash for PhantomData<T> {
    fn hash_into_xxhash3(&self, _hasher: &mut xxhash_rust::xxh3::Xxh3) {
        // hasher.update(b"::std::marker::PhantomData");
        // that may be bad?
        // do nothing now
    }
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
impl<T: XXHash3Hash + ?Sized> XXHash3Hash for &T {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        (**self).hash_into_xxhash3(hasher)
    }
}

/// Allow calling on &mut T
impl<T: XXHash3Hash + ?Sized> XXHash3Hash for &mut T {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        (**self).hash_into_xxhash3(hasher)
    }
}

impl XXHash3Hash for smol_str::SmolStr {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(b"::smol_str::SmolStr");
        self.len().hash_into_xxhash3(hasher);
        hasher.update(self.as_bytes());
    }
}

/// Allow calling on String
impl XXHash3Hash for String {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(b"::std::string::String");
        self.len().hash_into_xxhash3(hasher);
        hasher.update(self.as_bytes());
    }
}

impl<T: XXHash3Hash> XXHash3Hash for &[T] {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(b"::std::slice::Slice");
        self.len().hash_into_xxhash3(hasher);

        for value in self.iter() {
            value.hash_into_xxhash3(hasher);
        }
    }
}
impl<T: XXHash3Hash, const N: usize> XXHash3Hash for [T; N] {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(b"::std::array::Array");
        self.len().hash_into_xxhash3(hasher);

        for value in self.iter() {
            value.hash_into_xxhash3(hasher);
        }
    }
}

impl<'a, T: XXHash3Hash + Clone> XXHash3Hash for std::borrow::Cow<'a, T> {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        self.as_ref().hash_into_xxhash3(hasher);
    }
}

/// Allow calling on &str
impl XXHash3Hash for str {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(b"::std::str::Str");
        self.len().hash_into_xxhash3(hasher);
        hasher.update(self.as_bytes());
    }
}
/// Allow calling on PathBuf
impl XXHash3Hash for Utf8PathBuf {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(b"::camino::Utf8PathBuf");
        self.as_str().hash_into_xxhash3(hasher);
    }
}
/// Allow calling on Path
impl XXHash3Hash for Utf8Path {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(b"::camino::Utf8Path");
        self.as_str().hash_into_xxhash3(hasher);
    }
}
/// Allow calling on Rc<T>
impl<T: XXHash3Hash + ?Sized> XXHash3Hash for Rc<T> {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        (**self).hash_into_xxhash3(hasher)
    }
}
/// Allow calling on Arc<T>
impl<T: XXHash3Hash + ?Sized> XXHash3Hash for Arc<T> {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        (**self).hash_into_xxhash3(hasher)
    }
}
/// Allow calling on Box<T>
impl<T: XXHash3Hash + ?Sized> XXHash3Hash for Box<T> {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        (**self).hash_into_xxhash3(hasher)
    }
}

impl<T: XXHash3Hash + Sized + Ord + Eq> XXHash3Hash for Vec<T> {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(b"::std::vec::Vec");
        self.len().hash_into_xxhash3(hasher);

        for value in self.iter() {
            value.hash_into_xxhash3(hasher);
        }
    }
}

impl<T: XXHash3Hash, V: XXHash3Hash> XXHash3Hash for BTreeMap<T, V> {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(b"::std::collections::BTreeMap");
        self.len().hash_into_xxhash3(hasher);

        for pair in self.iter() {
            pair.0.hash_into_xxhash3(hasher);
            pair.1.hash_into_xxhash3(hasher);
        }
    }
}
impl<T: XXHash3Hash> XXHash3Hash for BTreeSet<T> {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(b"::std::collections::BTreeSet");
        self.len().hash_into_xxhash3(hasher);

        for pair in self.iter() {
            pair.hash_into_xxhash3(hasher);
        }
    }
}

impl<T: XXHash3Hash + Sized + Ord + std::hash::Hash + Eq, V: XXHash3Hash, S> XXHash3Hash
    for HashMap<T, V, S>
{
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(b"::std::collections::HashMap");
        self.len().hash_into_xxhash3(hasher);

        let mut pairs = self.iter().collect::<Vec<_>>();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));

        for pair in pairs.into_iter() {
            pair.0.hash_into_xxhash3(hasher);
            pair.1.hash_into_xxhash3(hasher);
        }
    }
}

impl<T: XXHash3Hash + Sized + Ord + std::hash::Hash + Eq, S> XXHash3Hash for HashSet<T, S> {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(b"::std::collections::HashSet");
        self.len().hash_into_xxhash3(hasher);

        let mut items = self.iter().collect::<Vec<_>>();
        items.sort();

        for pair in items.into_iter() {
            pair.hash_into_xxhash3(hasher);
        }
    }
}

/// Allow calling on Option<T>
///
/// If options it none, it will hash a tag byte 0u8 and the unit hash
/// If option is some, it will hash a tag byte 1u8 and the value's hash.
impl<T: XXHash3Hash + Sized> XXHash3Hash for Option<T> {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(b"::std::option::Option");
        match self {
            Some(value) => {
                hasher.update(&[1u8]); // Tag
                value.hash_into_xxhash3(hasher); // value
            }
            None => {
                hasher.update(&[0u8]); // Tag
                ().hash_into_xxhash3(hasher); // use unit as no value
            }
        };
    }
}

pub const UNIT_HASH_MAGIC: u128 = 0x0011_2233_4455_6677_8899_AABB_CCDD_EEFF;

impl XXHash3Hash for () {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(&UNIT_HASH_MAGIC.to_le_bytes());
    }
}

/// Allow calling on fixed-size numbers
///
/// See [the trait doc](XXHash3) for explanation.
macro_rules! impl_blake3_hash_for_fixed_numbers {
    ($($t:ty),*) => {
        $(
            impl XXHash3Hash for $t {
                fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
                    hasher.update(&self.to_le_bytes());
                }
            }
        )*
    }
}

impl_blake3_hash_for_fixed_numbers!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

impl XXHash3Hash for char {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        (*self as u32).hash_into_xxhash3(hasher);
    }
}

impl XXHash3Hash for f32 {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        let val = if self.is_nan() {
            0x7FC00000_u32 // f32::NaN is platform/compiler-dependent
        } else if *self == 0.0 {
            0_u32 // 统一 +0.0 和 -0.0
        } else {
            self.to_bits()
        };
        hasher.update(&val.to_le_bytes());
    }
}

impl XXHash3Hash for f64 {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        let val = if self.is_nan() {
            0x7FF8000000000000_u64 // f64::NaN is platform/compiler-dependent
        } else if *self == 0.0_f64 {
            0_u64 // 统一 +0.0 和 -0.0
        } else {
            self.to_bits()
        };
        hasher.update(&val.to_le_bytes());
    }
}

/// Allow calling on isize and usize
///
/// See [the trait doc](XXHash3) for explanation.
macro_rules! impl_blake3_hash_for_arch_numbers {
    ($($t:ty),*) => {
        $(
            impl XXHash3Hash for $t {
                fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
                    (*self as u64).hash_into_xxhash3(hasher)
                }
            }
        )*
    }
}

impl_blake3_hash_for_arch_numbers!(usize, isize);

impl XXHash3Hash for bool {
    fn hash_into_xxhash3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        let val = if *self { 1u8 } else { 0u8 };
        hasher.update(&[val]);
    }
}

#[derive(
    Clone,
    Debug,
    Hash,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Hash, Eq, PartialEq, PartialOrd, Ord))]
pub struct Hash {
    xxhash3: u128,
}

impl Hash {
    pub fn new(hash_value: u128) -> Self {
        Self {
            xxhash3: hash_value,
        }
    }

    pub fn get_le_bytes(&self) -> [u8; 16] {
        self.xxhash3.to_le_bytes()
    }

    pub fn get(&self) -> u128 {
        self.xxhash3
    }

    pub fn from_le_bytes(bytes: &[u8; 16]) -> Self {
        Self {
            xxhash3: u128::from_le_bytes(*bytes),
        }
    }

    pub fn to_hex(&self) -> arrayvec::ArrayString<32> {
        let mut s: arrayvec::ArrayString<32> = arrayvec::ArrayString::new();
        let mut buf = [0u8; 32];
        hex::encode_to_slice(self.get_le_bytes(), &mut buf).unwrap();
        unsafe {
            // skip check for performance
            s.push_str(std::str::from_utf8_unchecked(&buf));
        }
        s
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = TryFromSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self::from_le_bytes(value.try_into()?))
    }
}

impl std::ops::Deref for Hash {
    type Target = u128;

    fn deref(&self) -> &Self::Target {
        &self.xxhash3
    }
}

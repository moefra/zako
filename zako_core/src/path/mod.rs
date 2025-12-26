use camino::Utf8Path;
use phf::phf_set;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::string::String;
use thiserror::Error;
use zako_digest::blake3_hash::Blake3Hash;

use crate::intern::Interner;
use crate::path::interned::InternedNeutralPath;

pub mod interned;

type StackString<'a> = smallvec::SmallVec<[&'a str; 16]>;

/// From [Microsoft's document](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file)
///
/// Note that `¹`,`²`,`³` takes two bytes in UTF-8 encoding, but it's still a single character.
pub static WINDOWS_RESERVED: phf::Set<&'static str> = phf_set! {
    "CON" , "PRN" , "AUX" , "NUL" , "COM1" , "COM2" , "COM3" , "COM4" , "COM5" , "COM6",
    "COM7" , "COM8" , "COM9" , "LPT1" , "LPT2" , "LPT3" , "LPT4" , "LPT5" , "LPT6",
    "LPT7" , "LPT8" , "LPT9" , "COM¹" , "COM²" , "COM³" , "LPT¹" , "LPT²" , "LPT³"
};

/// NeutralPath is a neutral, normalized, relative file path representation.
///
/// It is designed to be used in cross-platform applications where file paths need to be handled.
///
/// - Always uses '/' as the separator
/// - Does not contain redundant `.` or `..` components, unless the path itself starts with `..` or is just `.`
/// - All paths are valid on both Unix and Windows
/// - Does not contain redundant '/' separators
/// - Does not contain absolute path prefixes
/// - Does not contain drive prefixes like C:
/// - Is a valid UTF-8 string
#[derive(
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    rkyv::Deserialize,
    rkyv::Serialize,
    rkyv::Archive,
    PartialOrd,
    Ord,
)]
#[serde(try_from = "String", into = "String")]
pub struct NeutralPath(String);

#[derive(Error, Debug)]
pub enum PathError {
    #[error("The file path is empty")]
    EmptyPath,
    #[error("Invalid path format: {0}")]
    InvalidFormat(&'static str),
    #[error("Invalid path character: {0}")]
    InvalidCharacter(&'static str),
    #[error("Invalid unicode data detected")]
    InvalidUnicodeData(),
    #[error(
        "Path is absolute, but a relative path is required for zmake::path::NeutralPath to construct or join"
    )]
    PathIsAbsolute(),
}

impl Display for NeutralPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for NeutralPath {
    fn default() -> Self {
        NeutralPath::from_path(".").unwrap()
    }
}

impl AsRef<str> for NeutralPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<Path> for NeutralPath {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl AsRef<NeutralPath> for NeutralPath {
    fn as_ref(&self) -> &NeutralPath {
        &self
    }
}

impl TryFrom<&str> for NeutralPath {
    type Error = PathError;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        NeutralPath::from_path(path)
    }
}

impl TryFrom<&Path> for NeutralPath {
    type Error = PathError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let path = path.to_str().ok_or(Self::Error::InvalidUnicodeData())?;
        NeutralPath::from_path(path)
    }
}

impl TryFrom<String> for NeutralPath {
    type Error = PathError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        NeutralPath::from_path(s)
    }
}

impl Into<String> for NeutralPath {
    fn into(self) -> String {
        self.0
    }
}

impl Into<PathBuf> for NeutralPath {
    fn into(self) -> PathBuf {
        PathBuf::from(self.0)
    }
}

impl std::fmt::Debug for NeutralPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NeutralPath({})", self.0)
    }
}

impl NeutralPath {
    fn check_if_absolute(part: &str) -> Result<(), PathError> {
        // check if starts with / or \ or \\
        if part.starts_with('/') || part.starts_with("\\") {
            return Err(PathError::PathIsAbsolute());
        }

        if let Some((left, _right)) = part.split_once(":") {
            // check if like C: or d:
            if left.chars().all(|c| c.is_ascii_alphabetic()) {
                return Err(PathError::PathIsAbsolute());
            }
        }

        Ok(())
    }

    fn check_path_name_is_valid(part: &str) -> Result<(), PathError> {
        // limitation from unix
        if part.contains('\0') {
            return Err(PathError::InvalidCharacter("\\0"));
        }

        // limitation from windows
        if part.contains("<") {
            return Err(PathError::InvalidCharacter("<"));
        }
        if part.contains(">") {
            return Err(PathError::InvalidCharacter(">"));
        }
        if part.contains(":") {
            return Err(PathError::InvalidCharacter(":"));
        }
        if part.contains("\"") {
            return Err(PathError::InvalidCharacter("\""));
        }
        if part.contains("|") {
            return Err(PathError::InvalidCharacter("|"));
        }
        if part.contains("?") {
            return Err(PathError::InvalidCharacter("?"));
        }
        if part.contains("*") {
            return Err(PathError::InvalidCharacter("*"));
        }

        if part.ends_with(" ") {
            return Err(PathError::InvalidFormat("space ` ` at the end"));
        }

        if part.ends_with(".") {
            return Err(PathError::InvalidFormat("dot `.` at the end"));
        }

        // NUL NUL.gzip NUL.tar.gz is all invalid
        let stem = part.split_once('.').map(|(l, _)| l).unwrap_or(part);

        let len = stem.len();
        // 3 for COM ...
        // 4 for COM1 ...
        // 5 for COM¹ ...
        if len == 3 || len == 4 || len == 5 {
            let mut buf = [0u8; 5];

            // 将 bytes 复制到 buffer 并转大写
            // 这里假设是 ASCII，因为 unicode 保留字很少见
            for (i, b) in stem.bytes().enumerate() {
                buf[i] = b.to_ascii_uppercase();
            }

            // 将 buffer 转为 &str (unsafe 是安全的，因为我们只处理了 ASCII)
            let upper_stem = std::str::from_utf8(&buf[0..len]).unwrap_or("");

            // 4. PHF 查表 (O(1))
            if WINDOWS_RESERVED.contains(upper_stem) {
                return Err(PathError::InvalidCharacter("Windows reserved name"));
            }
        }

        // limitation from my brain
        if part.contains("\'") {
            return Err(PathError::InvalidCharacter("\'"));
        }
        if part.contains("`") {
            return Err(PathError::InvalidCharacter("`"));
        }
        if part.contains("$") {
            return Err(PathError::InvalidCharacter("$"));
        }

        return Ok(());
    }

    fn internal_normalize(path: &str, check: bool) -> Result<NeutralPath, PathError> {
        let split = path.split(['/', '\\']);
        let mut components: StackString = StackString::new();

        for part in split {
            if part == "." || part.is_empty() {
                continue;
            } else if part == ".." {
                if let Some(last) = components.last() {
                    if *last == ".." {
                        components.push("..");
                    } else {
                        components.pop();
                    }
                } else {
                    components.push(part);
                }
            } else {
                if check {
                    Self::check_path_name_is_valid(part)?;
                }
                components.push(part);
            }
        }

        if components.is_empty() {
            components.push(".")
        }

        Ok(NeutralPath(components.join("/")))
    }

    fn checked_normalize(path: &str) -> Result<NeutralPath, PathError> {
        Self::internal_normalize(path, true)
    }

    fn unchecked_normalize(path: &NeutralPath) -> NeutralPath {
        // skip check for invalid path component
        Self::internal_normalize(&path.0, false).unwrap()
    }

    pub fn from_path<S: AsRef<Utf8Path>>(s: S) -> Result<Self, PathError> {
        let s = s.as_ref();

        Self::check_if_absolute(s.as_str())?;

        Self::checked_normalize(s.as_str())
    }

    pub fn current_dir() -> Self {
        NeutralPath::from_path(".").unwrap()
    }

    pub fn join<P: AsRef<str>>(&self, part: P) -> Result<Self, PathError> {
        let part_str = part.as_ref();
        Self::check_if_absolute(part_str)?;

        let mut all_parts: StackString = StackString::new();
        all_parts.push(&self.0);
        all_parts.push(part_str);

        NeutralPath::from_path(all_parts.join("/"))
    }

    pub fn join_all<P: AsRef<str>>(&self, parts: &[P]) -> Result<Self, PathError> {
        let mut all_parts: StackString = StackString::new();

        all_parts.push(&self.0);

        for part in parts {
            let part_str = part.as_ref();
            Self::check_if_absolute(part_str)?;
            all_parts.push(part_str);
        }

        NeutralPath::from_path(all_parts.join("/"))
    }

    pub fn parent(&self) -> Self {
        self.join("..").unwrap() // should not fail
    }

    pub fn filename(&self) -> Option<&str> {
        if let Some((_, filename)) = self.0.rsplit_once('/') {
            if filename == "." || filename == ".." {
                // in case of ../..
                None
            } else {
                Some(filename)
            }
        } else {
            if self.0 == "." || self.0 == ".." {
                None
            } else {
                Some(&self.0)
            }
        }
    }

    pub fn extname(&self) -> Option<&str> {
        if let Some(filename) = self.filename() {
            if let Some((_, ext)) = filename.rsplit_once('.') {
                Some(ext)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn normalize(&self) -> Self {
        Self::unchecked_normalize(self)
    }

    pub fn get_relative_path_to(&self, to: &NeutralPath) -> Option<NeutralPath> {
        let from_parts: StackString = self.0.split('/').collect();
        let to_parts: StackString = to.0.split('/').collect();

        let mut common_length = 0;
        let max_common_length = std::cmp::min(from_parts.len(), to_parts.len());

        while common_length < max_common_length
            && from_parts[common_length] == to_parts[common_length]
        {
            common_length += 1;
        }

        let mut relative_parts: StackString = StackString::new();

        for _ in common_length..from_parts.len() {
            relative_parts.push("..");
        }

        for part in &to_parts[common_length..] {
            relative_parts.push(part);
        }

        if relative_parts.is_empty() {
            relative_parts.push(".");
        }

        Some(NeutralPath(relative_parts.join("/")))
    }

    pub fn is_in_dir(&self, dir: &NeutralPath) -> bool {
        let relative = dir.get_relative_path_to(self);
        if let Some(rel_path) = relative {
            if rel_path.0.starts_with("..") {
                return false;
            }
            return true;
        }
        false
    }

    pub fn intern(
        &self,
        interner: &Interner,
    ) -> Result<interned::InternedNeutralPath, ::zako_interner::InternerError> {
        Ok(unsafe { InternedNeutralPath::from_raw(interner.get_or_intern(self)?) })
    }
}

impl Blake3Hash for NeutralPath {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(self.0.as_bytes());
    }
}

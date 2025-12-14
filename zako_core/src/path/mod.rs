use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::string::String;
use thiserror::Error;

use crate::intern::Interner;
use crate::path::interned::InternedNeutralPath;

pub mod interned;

type StackString<'a> = smallvec::SmallVec<[&'a str; 8]>;

/// NeutralPath 代表一个平台无关的、标准化的、相对的路径。
///
/// 内部保证：
///
/// - 总是使用 '/' 作为分隔符
/// - 不包含多余的 `.` 或 `..` 组件，除非路径本身就是 `..` 开头或者路径只有 `.`
/// - 所有路径在Unix或者Windows上都合法
/// - 不包含多余的/分隔符
/// - 不包含绝对路径前缀
/// - 不包含诸如C:之类的驱动器前缀
/// - 是有效的 UTF-8 字符串
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
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
        NeutralPath::new(".").unwrap()
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
        NeutralPath::new(path)
    }
}

impl TryFrom<&Path> for NeutralPath {
    type Error = PathError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let path = path.to_str().ok_or(Self::Error::InvalidUnicodeData())?;
        NeutralPath::new(path)
    }
}

impl TryFrom<String> for NeutralPath {
    type Error = PathError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        NeutralPath::new(s)
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

        // NUL NUL.gzip NUL.tar.gz is all invalid
        let strip = if let Some((left, _right)) = part.split_once(".") {
            left
        } else {
            part
        };

        let strip = strip.to_ascii_uppercase();

        match strip.as_str() {
            "CON" | "PRN" | "AUX" | "NUL" | "COM1" | "COM2" | "COM3" | "COM4" | "COM5" | "COM6"
            | "COM7" | "COM8" | "COM9" | "LPT1" | "LPT2" | "LPT3" | "LPT4" | "LPT5" | "LPT6"
            | "LPT7" | "LPT8" | "LPT9" | "COM¹" | "COM²" | "COM³" | "LPT¹" | "LPT²" | "LPT³" =>
            {
                return Err(PathError::InvalidCharacter(
                    "reserved name from windows(e.g. NUL COM or LPT¹",
                ));
            }
            _ => {}
        }

        // limitation from my thought
        if part.contains("\'") {
            return Err(PathError::InvalidCharacter("\'"));
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

    pub fn new<S: AsRef<str>>(s: S) -> Result<Self, PathError> {
        let s = s.as_ref();

        if s.is_empty() {
            return Err(PathError::EmptyPath);
        }

        Self::check_if_absolute(s)?;

        Self::checked_normalize(s)
    }

    pub fn current_dir() -> Self {
        NeutralPath::new(".").unwrap()
    }

    pub fn join<P: AsRef<str>>(&self, part: P) -> Result<Self, PathError> {
        let part_str = part.as_ref();
        Self::check_if_absolute(part_str)?;

        let mut all_parts: StackString = StackString::new();
        all_parts.push(&self.0);
        all_parts.push(part_str);

        NeutralPath::new(all_parts.join("/"))
    }

    pub fn join_all<P: AsRef<str>>(&self, parts: &[P]) -> Result<Self, PathError> {
        let mut all_parts: StackString = StackString::new();

        all_parts.push(&self.0);

        for part in parts {
            let part_str = part.as_ref();
            Self::check_if_absolute(part_str)?;
            all_parts.push(part_str);
        }

        NeutralPath::new(all_parts.join("/"))
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

    pub fn intern(&self, interner: &mut Interner) -> interned::InternedNeutralPath {
        unsafe { InternedNeutralPath::from_raw(interner.get_or_intern(self.as_ref())) }
    }
}

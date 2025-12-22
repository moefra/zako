use std::path::{Path, PathBuf};

use bitcode::{Decode, Encode};
use ignore::WalkState;
use serde::{Deserialize, Serialize};
use tracing::{Level, event};
use ts_rs::TS;
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    context::BuildContext,
    intern::{Internable, InternedString, Interner},
};

/// The pattern to match file paths.
///
/// A interesting fact is that `ignore` is used to implement the pattern matching,
///
/// and it use `globset` crate internally.
///
/// Meaning that the pattern syntax can be found at the [docs of globset](https://docs.rs/globset/latest/globset/)
#[derive(
    TS, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Decode, Encode,
)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "pattern.d.ts")]
pub struct Pattern {
    /// The pattern syntax can be found at the [docs of globset](https://docs.rs/globset/latest/globset/)
    #[serde(default)]
    pub patterns: Vec<String>,
    /// Whether to follow standard ignore files like `.gitignore`, `.ignore`, etc.
    ///
    /// Default is true.
    ///
    /// It's provided by [ignore](https://docs.rs/ignore/latest/ignore/struct.WalkBuilder.html#method.standard_filters).
    ///
    /// But `hidden` is individually controlled by `ignore_hidden_files` field.
    #[serde(default = "crate::consts::default_true")]
    pub following_ignore_files: bool,
    /// Whether to ignore hidden files (files or directories starting with a dot).
    ///
    /// Default is false.
    ///
    /// It's provided by [ignore](https://docs.rs/ignore/latest/ignore/struct.WalkBuilder.html#method.hidden)
    #[serde(default = "crate::consts::default_false")]
    pub ignore_hidden_files: bool,
}

impl Blake3Hash for Pattern {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        for pattern in &self.patterns {
            pattern.hash_into_blake3(hasher);
        }
        self.following_ignore_files.hash_into_blake3(hasher);
        self.ignore_hidden_files.hash_into_blake3(hasher);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InternedPattern {
    pub patterns: Vec<InternedString>,
    pub following_ignore_files: bool,
    pub ignore_hidden_files: bool,
}

impl Pattern {
    pub fn intern(self, context: &BuildContext) -> InternedPattern {
        let interner = context.interner();
        InternedPattern {
            patterns: self
                .patterns
                .into_iter()
                .map(|p| interner.get_or_intern(&p))
                .collect(),
            following_ignore_files: self.following_ignore_files,
            ignore_hidden_files: self.ignore_hidden_files,
        }
    }
}

impl InternedPattern {
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    pub fn resolve(&self, interner: &Interner) -> Pattern {
        Pattern {
            patterns: self
                .patterns
                .iter()
                .map(|p| interner.resolve(&p).to_string())
                .collect(),
            following_ignore_files: self.following_ignore_files,
            ignore_hidden_files: self.ignore_hidden_files,
        }
    }

    pub fn walk(
        &self,
        interner: &Interner,
        current: &Path,
        threads: usize,
    ) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut walker = ignore::WalkBuilder::new(current);

        walker.threads(threads);
        walker.standard_filters(self.following_ignore_files);
        walker.hidden(self.ignore_hidden_files);

        for pattern in &self.patterns {
            walker.add(interner.resolve(pattern));
        }

        let bag = orx_concurrent_bag::ConcurrentBag::new();
        let walker = walker.build_parallel();

        let bag_ref = &bag;

        walker.run(|| {
            return Box::new(|result| {
                match result {
                    Err(err) => {
                        event!(
                            Level::WARN,
                            "get an ignore parallel walker error,zako ignore it and continue: {}",
                            err
                        );
                    }
                    Ok(entry) => {
                        let path = entry.path().to_path_buf();
                        bag_ref.push(path);
                    }
                }
                return WalkState::Continue;
            });
        });

        let result = bag.into_inner().into_iter().collect::<Vec<PathBuf>>();

        Ok(result)
    }
}

impl Blake3Hash for InternedPattern {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        for pattern in &self.patterns {
            hasher.update(&pattern.as_u64().to_le_bytes());
        }
        hasher.update(&if self.following_ignore_files {
            [1u8]
        } else {
            [0u8]
        });
        hasher.update(&if self.ignore_hidden_files {
            [1u8]
        } else {
            [0u8]
        });
    }
}

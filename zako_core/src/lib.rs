#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/moefra/assets/refs/heads/main/favicon/android-chrome-512x512.png",
    html_logo_url = "https://raw.githubusercontent.com/moefra/assets/refs/heads/main/logo/zako-logo-light-512x512.png",
    issue_tracker_base_url = "https://github.com/moefra/zako/issues"
)]
//! zako-core is the core library of the zako build system.
//!
//! Most code and docs are here. The crate `zako-cli` provides some useful command line documents.
//!
//! For Chinese contributors:
//! 最好在代码/文档中使用英文，因为这样可以让更多人受益。
//!
//! The are five types file in zako build system:
//!
//! - library file(`*.ts`): those file can be shared between other files. They can only import other library files and core built-in module like `zako:core`;
//! - script file(`*.script.ts`): those file can be used to write custom scripts. They can do anything and access `node:xxx`(or `Bun`,`Deno` object) modules,but they can not access zako's built-in module.
//! - project manifest file(`zako.json5`): those file is used to define project metadata like name,version,dependencies etc. It can not import any module.
//! - project root(`zako.ts`): those file is used to define a project.It is usually placed in the project root.It export build,rule and toolchain file. They can only import library files,core built-in module and `zako:project` module.
//! - build file(`BUILD.ts`): "Embrace the industry holy grail: BUILD.ts — as God intended." those file is used to define build targets.It is the most common file. They can only import library files,core built-in module and `zako:build` module.
//! - rule file(`*.rule.ts`): those file is used to define build rules.They can not access to system,they just get source file set and configuration from build file,process and convert configuration, access abstract toolchain. They can only import library files,core built-in module and `zako:rule` module.
//! - toolchain file(`*.toolchain.ts`): those file is used to define build tools.They can access to system,but they can only get input from rule files. They can only import library files,core built-in module and `zako:toolchain`
//!
//! Those file is name rule is used by `tsconfig.json` file to provide type checking and code completion.
//!
//! For zako,it should not rely on file suffix to determine file type. And no file can escape check regardless their name.
//!
//! An faster way is that, if a file is under `scripts` directory,it is treated as script file(In `tsconfig.json`).

use std::{clone, path::PathBuf};

use ignore::WalkState;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{Level, event};
use ts_rs::TS;
pub mod access_control;
pub mod author;
pub mod build_constants;
pub mod builtin;
mod cas;
mod cas_server;
pub mod config;
pub mod context;
pub mod dependency;
pub mod engine;
mod error;
mod extension;
pub mod file_finder;
pub mod fs;
pub mod id;
pub mod intern;
mod local_cas;
mod make_builtin;
pub mod mirror;
pub mod package;
pub mod path;
pub mod pattern;
mod platform;
pub mod project;
pub mod project_resolver;
pub mod sandbox;
pub mod socket_address;
pub mod target;
mod tool;
mod transformer;
mod transport_server;
pub mod v8error;
pub mod v8utils;
pub mod version_extractor;
mod zako_module_loader;

/// The result of iteration of this map is not ordered.
///
/// Please do not rely on any specific order.
pub type FastMap<K, V> = ::dashmap::DashMap<K, V, ::ahash::RandomState>;

/// The result of iteration of this map is not ordered.
///
/// Please do not rely on any specific order.
pub type FastSet<K> = ::dashmap::DashSet<K, ::ahash::RandomState>;

/// project file name.see root document for details
pub static SCRIPT_FILE_SUFFIX: &str = ".script.ts";

/// project file name.see root document for details
pub static LIBRARY_FILE_SUFFIX: &str = ".ts";

/// definition of project.see root document for details
pub static PROJECT_MANIFEST_FILE_NAME: &str = "zako.json5";

/// project file name.see root document for details
pub static PROJECT_SCRIPT_FILE_NAME: &str = "zako.ts";

/// build file name.see root document for details
pub static BUILD_FILE_NAME: &str = "BUILD.ts";

/// rule file suffix.see root document for details
pub static RULE_FILE_SUFFIX: &str = ".rule.ts";

/// toolchain file suffix.see root document for details
pub static TOOLCHAIN_FILE_SUFFIX: &str = ".toolchain.ts";

/// The type of zako build system file.
/// see root document for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// [PROJECT_FILE_NAME]
    Project,
    /// [BUILD_FILE_NAME]
    Build,
    /// [RULE_FILE_SUFFIX]
    Rule,
    /// [TOOLCHAIN_FILE_SUFFIX]
    Toolchain,
    /// [SCRIPT_FILE_SUFFIX]
    Script,
    /// [LIBRARY_FILE_SUFFIX]
    Library,
}

/// The pattern to match file paths.
#[derive(TS, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
#[ts(export, export_to = "pattern.d.ts")]
pub enum Pattern {
    /// Same as [Pattern::Glob] with empty excludes.
    List(Vec<String>),
    /// This will walk with prejudice. Including ignore files in .gitignore and add file suffix automatically.
    ///
    /// [Pattern::Glob::auto_filename] will auto add filename to matched set,like if you give "src/**",and filename is "zako.ts",it will match "src/**/zako.ts".  Default is true.
    ///
    /// [Pattern::Glob::auto_ignore] will respect ignore files like .gitignore automatically. Default is true.
    Glob {
        #[serde(default)]
        includes: Vec<String>,
        #[ts(optional)]
        #[serde(default)]
        excludes: Option<Vec<String>>,
        #[ts(optional)]
        #[serde(default)]
        auto_filename: Option<bool>,
        #[ts(optional)]
        #[serde(default)]
        auto_ignore: Option<bool>,
    },
    /// This will match exactly what you give. No glob support.
    Exact { files: Vec<String> },
}

impl Pattern {
    pub fn is_empty(&self) -> bool {
        match self {
            Pattern::List(v) => v.is_empty(),
            Pattern::Glob { includes, .. } => includes.is_empty(),
            Pattern::Exact { files } => files.is_empty(),
        }
    }

    pub fn walk(
        &self,
        filename: Option<&str>,
        mut current: PathBuf,
    ) -> Result<Vec<PathBuf>, std::io::Error> {
        current = current.canonicalize()?;

        let (includes, excludes, auto_filename, auto_ignore) = match self {
            Pattern::List(file_list) => (file_list.clone(), vec![], true, true),
            Pattern::Glob {
                includes,
                excludes,
                auto_filename,
                auto_ignore,
            } => {
                let mut incs: Vec<String> = includes.clone();
                let mut excludes: Vec<String> = excludes.clone().unwrap_or_default();
                (
                    incs,
                    excludes.clone(),
                    auto_filename.unwrap_or(true),
                    auto_ignore.unwrap_or(true),
                )
            }
            Pattern::Exact { files } => {
                return Ok(files
                    .clone()
                    .into_iter()
                    .map(|f| current.join(f).canonicalize())
                    .collect::<Result<Vec<PathBuf>, std::io::Error>>()?);
            }
        };

        let mut walker = ignore::WalkBuilder::new(current);

        walker.standard_filters(auto_ignore);

        for include in includes {
            if let Some(filename) = filename {
                walker.add(format!("{}/{}", include, filename));
            }
        }
        let filename = {
            if auto_filename && let Some(filename) = filename {
                Some(filename)
            } else {
                None
            }
        };
        for exclude in excludes {
            if let Some(filename) = filename {
                walker.add(format!("!{}/{}", exclude, filename));
            }
        }

        let bag = orx_concurrent_bag::ConcurrentBag::new();
        let walker = walker.build_parallel();

        let bag_ref = &bag;

        walker.run(|| {
            return Box::new(|result| {
                match result {
                    Err(err) => {
                        event!(Level::WARN, "get an ignore parallel walker error: {}", err);
                    }
                    Ok(entry) => {
                        let mut path = entry.path().to_string_lossy().to_string();
                        if let Some(filename) = &filename {
                            path = format!("{}/{}", path, filename);
                        }
                        bag_ref.push(PathBuf::from(path));
                    }
                }
                return WalkState::Continue;
            });
        });

        let result = bag.into_inner().into_iter().collect::<Vec<PathBuf>>();

        Ok(result)
    }
}

pub mod protobuf {
    pub mod fs {
        tonic::include_proto!("zako.v1.fs");
    }

    pub mod net {
        tonic::include_proto!("zako.v1.net");
    }

    pub mod cas {
        tonic::include_proto!("zako.v1.cas");
    }

    pub mod transport {
        tonic::include_proto!("zako.v1.transport");
    }
}

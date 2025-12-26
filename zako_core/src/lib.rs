#![doc(
    html_favicon_url = "https://raw.githubusercontent.com/moefra/assets/refs/heads/main/favicon/android-chrome-512x512.png",
    html_logo_url = "https://raw.githubusercontent.com/moefra/assets/refs/heads/main/favicon/favicon-32x32.png",
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
pub mod access_control;
pub mod author;
pub mod blob_handle;
pub mod blob_range;
pub mod build_constants;
pub mod builtin;
pub mod cas;
pub mod cas_server;
pub mod cas_store;
pub mod compute;
pub mod computer;
pub mod config;
pub mod config_value;
pub mod consts;
pub mod context;
pub mod engine;
pub mod error;
pub mod extension;
pub mod file_finder;
pub mod fs;
pub mod global_state;
pub mod id;
pub mod intern;
pub mod link;
pub mod local_cas;
mod make_builtin;
pub mod module_loader;
pub mod node;
pub mod package;
pub mod package_source;
pub mod path;
pub mod pattern;
pub mod persistent;
pub mod project;
pub mod project_resolver;
pub mod resource;
pub mod sandbox;
pub mod socket_address;
pub mod target;
pub mod tests;
pub mod tool;
pub mod transformer;
pub mod transport_server;
pub mod v8error;
pub mod v8platform;
pub mod v8utils;
pub mod version_extractor;
pub mod worker;

/// The result of iteration of this map is not ordered.
///
/// Please do not rely on any specific order.
pub type FastMap<K, V> = ::dashmap::DashMap<K, V, ::ahash::RandomState>;

/// The result of iteration of this map is not ordered.
///
/// Please do not rely on any specific order.
pub type FastSet<K> = ::dashmap::DashSet<K, ::ahash::RandomState>;

/// A fast cache implementation.
pub type FastCache<K, V> = ::moka::future::Cache<K, V, ::ahash::RandomState>;

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

    pub mod range {
        tonic::include_proto!("zako.v1.range");
    }
}

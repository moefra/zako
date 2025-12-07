//! Built-in module name rule:
//!
//! A module must be a simple name like `console`, assign it as $name.
//!
//! Then deno extension name will `zako_$name`
//!
//! The exported js module name will be `zako:$name`
pub mod console;
pub mod core;
pub mod global;
pub mod project;
pub mod rt;
pub mod semver;
pub mod syscall;

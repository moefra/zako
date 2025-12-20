/// project file name.see [crate] documents for details.
pub static SCRIPT_FILE_SUFFIX: &str = ".script.ts";

/// project file name.see [crate] documents for details.
pub static LIBRARY_FILE_SUFFIX: &str = ".ts";

/// definition of project.see [crate] documents for details.
pub static PROJECT_MANIFEST_FILE_NAME: &str = "zako.toml";

/// project file name.see [crate] documents for details.
pub static PROJECT_SCRIPT_FILE_NAME: &str = "zako.ts";

/// build file name.see [crate] documents for details.
pub static BUILD_FILE_NAME: &str = "BUILD.ts";

/// rule file suffix.see [crate] documents for details.
pub static RULE_FILE_SUFFIX: &str = ".rule.ts";

/// toolchain file suffix.see [crate] documents for details.
pub static TOOLCHAIN_FILE_SUFFIX: &str = ".toolchain.ts";

// TODO: it seems es2026 means esnext. switch to es2025 once they release it.
// Issue URL: https://github.com/moefra/zako/issues/12
/// The target version for transpiling TypeScript code.
pub static TRANSPILE_TARGET: &str = "es2026";

/// A function that returns true
#[inline]
pub const fn default_true() -> bool {
    true
}

/// A function that returns false
#[inline]
pub const fn default_false() -> bool {
    false
}

/// The type of zako build system file.
/// see [crate] documents for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// [PROJECT_MANIFEST_FILE_NAME]
    ProjectManifest,
    /// [PROJECT_SCRIPT_FILE_NAME]
    ProjectScript,
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

/// The type of V8 context used in Zako build system.
///
/// see [crate] documents for details.
///
/// Different context types have different permissions and capabilities.
///
/// The `Script` was not processed by zako. It usually create a new bun process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V8ContextType {
    /// Enable `zako:project` for
    ///
    /// [PROJECT_MANIFEST_FILE_NAME]
    ///
    /// And
    ///
    /// [PROJECT_SCRIPT_FILE_NAME]
    Project,
    /// Enable `zako:build` for
    ///
    /// [BUILD_FILE_NAME]
    Build,
    /// Enable `zako:rule` for
    ///
    /// [RULE_FILE_SUFFIX]
    Rule,
    /// Enable `zako:toolchain` for
    ///
    /// [TOOLCHAIN_FILE_SUFFIX]
    Toolchain,
}

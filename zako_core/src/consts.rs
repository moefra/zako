/// project file name.see [crate] documents for details.
pub static SCRIPT_FILE_SUFFIX: &str = ".script.ts";

/// project file name.see [crate] documents for details.
pub static LIBRARY_FILE_SUFFIX: &str = ".ts";

/// definition of project.see [crate] documents for details.
pub static PACKAGE_MANIFEST_FILE_NAME: &str = "zako.toml";

/// project file name.see [crate] documents for details.
pub static PACKAGE_SCRIPT_FILE_NAME: &str = "zako.ts";

/// build file name.see [crate] documents for details.
pub static BUILD_FILE_NAME: &str = "BUILD.ts";

/// rule file suffix.see [crate] documents for details.
pub static RULE_FILE_SUFFIX: &str = ".rule.ts";

/// toolchain file suffix.see [crate] documents for details.
pub static TOOLCHAIN_FILE_SUFFIX: &str = ".toolchain.ts";

/// config file suffix.see [crate] documents for details.
pub static CONFIG_FILE_SUFFIX: &str = ".config.ts";

// TODO: it seems es2026 means esnext. switch to es2025 once they release it.
// Issue URL: https://github.com/moefra/zako/issues/12
/// The target version for transpiling TypeScript code.
pub static TRANSPILE_TARGET: &str = "es2026";

/// By default, we mount the configuration to the `config` label path.
///
/// See [crate::package::Package::mount_config] for more details.
pub static DEFAULT_CONFIGURATION_MOUNT_POINT: &str = "config";

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
    /// [PACKAGE_MANIFEST_FILE_NAME]
    PackageManifest,
    /// [PACKAGE_SCRIPT_FILE_NAME]
    PackageScript,
    /// [BUILD_FILE_NAME]
    Build,
    /// [CONFIG_FILE_SUFFIX]
    Config,
    /// [RULE_FILE_SUFFIX]
    Rule,
    /// [TOOLCHAIN_FILE_SUFFIX]
    Toolchain,
    /// [SCRIPT_FILE_SUFFIX]
    Script,
    /// [LIBRARY_FILE_SUFFIX]
    Library,
}

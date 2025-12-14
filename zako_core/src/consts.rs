/// project file name.see [crate] documents for details.
pub static SCRIPT_FILE_SUFFIX: &str = ".script.ts";

/// project file name.see [crate] documents for details.
pub static LIBRARY_FILE_SUFFIX: &str = ".ts";

/// definition of project.see [crate] documents for details.
pub static PROJECT_MANIFEST_FILE_NAME: &str = "zako.json5";

/// project file name.see [crate] documents for details.
pub static PROJECT_SCRIPT_FILE_NAME: &str = "zako.ts";

/// build file name.see [crate] documents for details.
pub static BUILD_FILE_NAME: &str = "BUILD.ts";

/// rule file suffix.see [crate] documents for details.
pub static RULE_FILE_SUFFIX: &str = ".rule.ts";

/// toolchain file suffix.see [crate] documents for details.
pub static TOOLCHAIN_FILE_SUFFIX: &str = ".toolchain.ts";

/// A function that returns true
pub fn default_true() -> bool {
    true
}

/// A function that returns false
pub fn default_false() -> bool {
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

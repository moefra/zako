use crate::{
    config::ResolvedConfiguration, configured_project::ConfiguredPackage, package::Package,
};

/// The type of V8 context used in Zako build system.
///
/// see [crate] documents for details.
///
/// Different context types have different permissions and capabilities.
///
/// The `Script` was not processed by zako. It usually create a new bun process.
#[derive(Debug, Clone)]
pub enum V8ContextInput {
    /// Enable `zako:package` for
    ///
    /// [crate::consts::PACKAGE_SCRIPT_FILE_NAME]
    Package { package: ConfiguredPackage },
    /// Enable `zako:build` for
    ///
    /// [crate::consts::BUILD_FILE_NAME]
    Build { package: Package },
    /// Enable `zako:rule` for
    ///
    /// [crate::consts::RULE_FILE_SUFFIX]
    Rule,
    /// Enable `zako:toolchain` for
    ///
    /// [crate::consts::TOOLCHAIN_FILE_SUFFIX]
    Toolchain,
    /// Enable `zako:config` for
    ///
    /// [crate::consts::CONFIG_FILE_SUFFIX]
    Config {
        package: Package,
        allow_access_system: bool,
    },
}

#[derive(Debug, Clone)]
pub enum V8ContextOutput {
    /// Enable `zako:package` for
    ///
    /// [crate::consts::PACKAGE_SCRIPT_FILE_NAME]
    Package { package: ConfiguredPackage },
    /// Enable `zako:build` for
    ///
    /// [crate::consts::BUILD_FILE_NAME]
    Build { package: Package },
    /// Enable `zako:rule` for
    ///
    /// [crate::consts::RULE_FILE_SUFFIX]
    Rule,
    /// Enable `zako:toolchain` for
    ///
    /// [crate::consts::TOOLCHAIN_FILE_SUFFIX]
    Toolchain,
    /// Enable `zako:config` for
    ///
    /// [crate::consts::CONFIG_FILE_SUFFIX]
    Config {
        configuration: ResolvedConfiguration,
    },
}

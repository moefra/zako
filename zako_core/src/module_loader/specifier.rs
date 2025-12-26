use std::{fmt::Display, path::PathBuf};

use url::Url;

/// If a module start with this, it will seem as a builtin module.
///
/// The builtin module is provided by Zako in memory.
pub static BUILTIN_MODULE_PREFIX: &str = "zako:";
/// The schema of builtin module.
pub static BUILTIN_MODULE_SCHEMA: &str = "zako";

/// If a module start with this, it will seem as a memory module.
///
/// The memory module can not load any other module.
pub static MEMORY_MODULE_PREFIX: &str = "__zako_memory:";
/// The schema of memory module.
pub static MEMORY_MODULE_SCHEMA: &str = "zako-memory";

/// If a module start with this, it will seem as an import map module.
pub static IMPORT_MAP_MODULE_PREFIX: &str = "@";
/// The schema of import map module.
pub static IMPORT_MAP_MODULE_SCHEMA: &str = "zako-import-map";

/// The schema of file module.
pub static FILE_MODULE_SCHEMA: &str = "file";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ModuleType {
    File,
    Builtin,
    Memory,
    ImportMap,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModuleSpecifier {
    pub url: Url,
    pub module_type: ModuleType,
}

impl ModuleSpecifier {
    pub fn new(url: Url, module_type: ModuleType) -> Self {
        Self { url, module_type }
    }

    pub fn new_file_module(file: &PathBuf) -> Result<Self, super::ModuleLoadError> {
        Ok(Self::new(
            Url::parse(&format!(
                "{}://{}",
                FILE_MODULE_SCHEMA,
                file.to_string_lossy().to_string()
            ))?,
            ModuleType::File,
        ))
    }

    pub fn from(module_specifier: &str) -> Result<Self, super::ModuleLoadError> {
        match Url::try_from(module_specifier) {
            Ok(url) => {
                let scheme = url.scheme();
                if scheme.eq(BUILTIN_MODULE_SCHEMA) {
                    return Ok(ModuleSpecifier::new(url, ModuleType::Builtin));
                } else if scheme.eq(MEMORY_MODULE_SCHEMA) {
                    return Ok(ModuleSpecifier::new(url, ModuleType::Memory));
                } else if scheme.eq(IMPORT_MAP_MODULE_SCHEMA) {
                    return Ok(ModuleSpecifier::new(url, ModuleType::ImportMap));
                } else if scheme.eq(FILE_MODULE_SCHEMA) {
                    return Ok(ModuleSpecifier::new(url, ModuleType::File));
                }
            }
            Err(_) => {}
        }

        let (raw_url, module_type) = {
            if let Some(specifier) = module_specifier.strip_prefix(BUILTIN_MODULE_PREFIX) {
                (
                    format!("{}:{}", BUILTIN_MODULE_SCHEMA, specifier),
                    ModuleType::Builtin,
                )
            } else if let Some(specifier) = module_specifier.strip_prefix(MEMORY_MODULE_PREFIX) {
                (
                    format!("{}:{}", MEMORY_MODULE_SCHEMA, specifier),
                    ModuleType::Memory,
                )
            } else if let Some(specifier) = module_specifier.strip_prefix(IMPORT_MAP_MODULE_PREFIX)
            {
                (
                    format!("{}:{}", IMPORT_MAP_MODULE_SCHEMA, specifier),
                    ModuleType::ImportMap,
                )
            } else {
                (
                    format!("{}://{}", FILE_MODULE_SCHEMA, module_specifier),
                    ModuleType::File,
                )
            }
        };

        let url = Url::parse(&raw_url).map_err(|err| {
            eyre::Report::new(err).wrap_err(format!(
                "Failed to parse url `{}` (detected module type: {:?})",
                raw_url, module_type
            ))
        })?;

        Ok(ModuleSpecifier::new(url, module_type))
    }
}

impl AsRef<Url> for ModuleSpecifier {
    fn as_ref(&self) -> &Url {
        &self.url
    }
}

impl Display for ModuleSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(module: `{}`, type: {:?})", self.url, self.module_type)
    }
}

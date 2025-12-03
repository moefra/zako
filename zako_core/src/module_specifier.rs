use std::fmt::Display;
use std::path::PathBuf;

pub static MEMORY_MODULE_PREFIX: &'static str = "__ZMAKE_MEMORY_MODULE_";

pub static IMPORT_MAP_MODULE_PREFIX: &'static str = "@";

pub static BUILTIN_MODULE_PREFIX: &'static str = "zmake:";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ModuleSpecifier {
    /// Built-in module,start with `BUILTIN_MODULE_PREFIX`
    ///
    /// The string does not contain `BUILTIN_MODULE_PREFIX`
    Builtin(String),
    /// Memory module,start with `MEMORY_MODULE_PREFIX`
    ///
    /// The string does not contain `MEMORY_MODULE_PREFIX`
    Memory(String),
    /// The path refer to the target file.
    ///
    /// The path is in the sandbox and access this path is safe.
    File(PathBuf),
    /// Import map module,start with `IMPORT_MAP_MODULE_PREFIX`
    ///
    /// The string does not contain `IMPORT_MAP_MODULE_PREFIX`
    ImportMap(String),
}

impl From<&str> for ModuleSpecifier {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

impl From<String> for ModuleSpecifier {
    fn from(mut s: String) -> Self {
        if s.starts_with(BUILTIN_MODULE_PREFIX) {
            ModuleSpecifier::Builtin(s.split_off(BUILTIN_MODULE_PREFIX.len()))
        } else if s.starts_with(MEMORY_MODULE_PREFIX) {
            ModuleSpecifier::Memory(s.split_off(MEMORY_MODULE_PREFIX.len()))
        } else if s.starts_with(IMPORT_MAP_MODULE_PREFIX) {
            ModuleSpecifier::ImportMap(s.split_off(IMPORT_MAP_MODULE_PREFIX.len()))
        } else {
            ModuleSpecifier::File(PathBuf::from(s))
        }
    }
}

impl Into<String> for ModuleSpecifier {
    fn into(self) -> String {
        self.with_prefix()
    }
}

impl Display for ModuleSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.with_prefix())
    }
}

impl AsRef<ModuleSpecifier> for ModuleSpecifier {
    fn as_ref(&self) -> &ModuleSpecifier {
        &self
    }
}

impl ModuleSpecifier {
    pub fn with_prefix(&self) -> String {
        match self {
            ModuleSpecifier::Builtin(s) => format!("{}{}", BUILTIN_MODULE_PREFIX, s),
            ModuleSpecifier::Memory(s) => format!("{}{}", MEMORY_MODULE_PREFIX, s),
            ModuleSpecifier::ImportMap(s) => format!("{}{}", IMPORT_MAP_MODULE_PREFIX, s),
            ModuleSpecifier::File(p) => {
                format!("{}", p.to_string_lossy().to_string())
            }
        }
    }
}

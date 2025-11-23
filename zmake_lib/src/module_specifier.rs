use std::fmt::Display;
use std::path::PathBuf;

pub static MEMORY_MODULE_PREFIX: &str = "__ZMAKE_MEMORY_MODULE_";

pub static IMPORT_MAP_MODULE_PREFIX: &str = "@";

pub static BUILTIN_MODULE_PREFIX: &str = "zmake:";

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum ModuleSpecifier{
    Builtin(String),
    Memory(String),
    File(PathBuf),
    ImportMap(String),
}

impl From<&str> for ModuleSpecifier {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

impl From<String> for ModuleSpecifier {
    fn from(s: String) -> Self {
        if s.starts_with(BUILTIN_MODULE_PREFIX) {
            ModuleSpecifier::Builtin(s)
        } else if s.starts_with(MEMORY_MODULE_PREFIX) {
            ModuleSpecifier::Memory(s)
        } else if s.starts_with(IMPORT_MAP_MODULE_PREFIX) {
            ModuleSpecifier::ImportMap(s)
        } else {
            ModuleSpecifier::File(PathBuf::from(s))
        }
    }
}

impl Into<String> for ModuleSpecifier {
    fn into(self) -> String {
        match self {
            ModuleSpecifier::Builtin(s) => s,
            ModuleSpecifier::Memory(s) => s,
            ModuleSpecifier::ImportMap(s) => s,
            ModuleSpecifier::File(p) => p.to_string_lossy().to_string(),
        }
    }
}

impl Display for ModuleSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <Self as Into<String>>::into(self.clone()))
    }
}

impl ModuleSpecifier {
    pub fn prefix_trimmed(&self)->String{
        match self {
            ModuleSpecifier::Builtin(s) => s.trim_start_matches(BUILTIN_MODULE_PREFIX).to_string(),
            ModuleSpecifier::Memory(s) => s.trim_start_matches(MEMORY_MODULE_PREFIX).to_string(),
            ModuleSpecifier::ImportMap(s) => s.trim_start_matches(IMPORT_MAP_MODULE_PREFIX).to_string(),
            ModuleSpecifier::File(p) => p.to_string_lossy().to_string(),
        }
    }
}

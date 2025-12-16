use crate::sandbox::SandboxRef;
use crate::transformer::transform_typescript;
use crate::zako_module_loader::ModuleType::ImportMap;
use ahash::{AHasher, HashSet};
use deno_core::JsRuntime;
use deno_core::ModuleLoadOptions;
use deno_core::ModuleLoadReferrer;
use deno_core::ModuleLoadResponse;
use deno_core::ModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSourceCode;
use deno_core::ResolutionKind;
use deno_core::RuntimeOptions;
use deno_core::error::{CoreError, ModuleLoaderError};
use deno_core::resolve_import;
use deno_core::resolve_path;
use deno_error::JsErrorBox;
use parking_lot::RwLock;
use serde::de;
use sha2::digest::typenum::op;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::Display;
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tracing::trace_span;
use url::{ParseError, Url};

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

#[derive(Error, Debug, deno_error::JsError)]
#[class(generic)]
pub enum ModuleLoadError {
    #[error("Get an url parse error:{0}")]
    UrlParseError(#[from] ParseError),
}

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

    pub fn new_file_module(file: &PathBuf) -> Result<Self, ModuleLoadError> {
        Ok(Self::new(
            Url::from_str(&format!(
                "{}://{}",
                FILE_MODULE_SCHEMA,
                file.to_string_lossy().to_string()
            ))?,
            ModuleType::File,
        ))
    }

    pub fn from(module_specifier: &str) -> Result<Self, ModuleLoadError> {
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

        if let Some(specifier) = module_specifier.strip_prefix(BUILTIN_MODULE_PREFIX) {
            let url = Url::from_str(&format!("{}:{}", BUILTIN_MODULE_SCHEMA, specifier))?;
            Ok(ModuleSpecifier::new(url, ModuleType::File))
        } else if let Some(specifier) = module_specifier.strip_prefix(MEMORY_MODULE_PREFIX) {
            let url = Url::from_str(&format!("{}:{}", MEMORY_MODULE_SCHEMA, specifier))?;
            Ok(ModuleSpecifier::new(url, ModuleType::File))
        } else if let Some(specifier) = module_specifier.strip_prefix(IMPORT_MAP_MODULE_PREFIX) {
            let url = Url::from_str(&format!("{}:{}", IMPORT_MAP_MODULE_SCHEMA, specifier))?;
            Ok(ModuleSpecifier::new(url, ModuleType::File))
        } else {
            let url = Url::from_str(&format!("{}://{}", FILE_MODULE_SCHEMA, module_specifier))?;
            Ok(ModuleSpecifier::new(url, ModuleType::File))
        }
    }
}

impl AsRef<Url> for ModuleSpecifier {
    fn as_ref(&self) -> &Url {
        &self.url
    }
}

impl Display for ModuleSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

pub type SourceMapStore =
    Arc<RwLock<std::collections::HashMap<String, Vec<u8>, ahash::RandomState>>>;

pub type LoadedSourceSets = Arc<RwLock<std::collections::HashSet<PathBuf, ahash::RandomState>>>;

#[derive(Debug, Clone)]
pub struct LoaderOptions {
    pub transpile_suffix: Vec<String>,
}

impl Default for LoaderOptions {
    fn default() -> Self {
        Self {
            transpile_suffix: vec!["ts".into(), "mts".into()],
        }
    }
}

impl LoaderOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub struct ZakoModuleLoader {
    source_maps: SourceMapStore,
    loaded_source_sets: LoadedSourceSets,
    options: LoaderOptions,
}

impl ZakoModuleLoader {
    pub fn new(options: LoaderOptions) -> Self {
        Self {
            source_maps: Arc::new(RwLock::new(std::collections::HashMap::with_hasher(
                ahash::RandomState::new(),
            ))),
            loaded_source_sets: Arc::new(RwLock::new(HashSet::with_hasher(
                ahash::RandomState::new(),
            ))),
            options,
        }
    }
}

impl ModuleLoader for ZakoModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        kind: ResolutionKind,
    ) -> Result<deno_core::ModuleSpecifier, ModuleLoaderError> {
        let _span = trace_span!(
            "resolve module",
            referrer,
            specifier,
            kind = format!("{:?}", kind)
        )
        .entered();

        let referrer = ModuleSpecifier::from(referrer).map_err(JsErrorBox::from_err)?;
        let specifier = ModuleSpecifier::from(specifier).map_err(JsErrorBox::from_err)?;
        let builtin_specifier: Result<deno_core::ModuleSpecifier, ModuleLoaderError> =
            Ok(specifier.url.clone());

        match referrer.module_type {
            ModuleType::File => match specifier.module_type {
                ModuleType::File => {
                    let resolved = resolve_import(&specifier.to_string(), &referrer.to_string())
                        .map_err(JsErrorBox::from_err)?;
                    Ok(resolved)
                }
                ModuleType::Builtin => Ok(builtin_specifier?),
                ModuleType::Memory => Err(JsErrorBox::generic(format!(
                    "File module can not import memory module, referrer:`{:?}` specifier:`{:?}`",
                    referrer, specifier
                ))),
                ModuleType::ImportMap => {
                    todo!()
                }
            },
            ModuleType::Builtin => {
                return if ModuleType::Builtin == specifier.module_type {
                    Ok(builtin_specifier?)
                } else {
                    Err(JsErrorBox::generic(format!(
                        "Builtin module can only import builtin module, referrer:`{:?}` specifier:`{:?}`",
                        referrer, specifier
                    )))
                };
            }
            ModuleType::Memory => match specifier.module_type {
                ModuleType::File => Err(JsErrorBox::generic(format!(
                    "Memory module can not request file module: referer `{:?}` specifier:`{:?}`",
                    referrer, specifier
                ))),
                ModuleType::Builtin => Ok(builtin_specifier?),
                ModuleType::Memory => Err(JsErrorBox::generic(format!(
                    "Memory module can not import memory module, referrer:`{:?}` specifier:`{:?}`",
                    referrer, specifier
                ))),
                ModuleType::ImportMap => {
                    todo!()
                }
            },
            ModuleType::ImportMap => {
                todo!()
            }
        }
    }

    fn load(
        &self,
        module_specifier: &deno_core::ModuleSpecifier,
        maybe_referrer: Option<&ModuleLoadReferrer>,
        options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        ModuleLoadResponse::Sync((move || {
            let _span = trace_span!(
                "load module",
                maybe_referrer = format!("{:?}", maybe_referrer),
                module_specifier = &module_specifier.to_string(),
                options = format!(
                    "{{is_dynamic_import: {}, is_synchronous: {}, requested_module_type: {}}}",
                    options.is_dynamic_import,
                    options.is_synchronous,
                    options.requested_module_type
                )
            )
            .entered();

            let path = module_specifier
                .to_file_path()
                .map_err(|_| JsErrorBox::generic("Only file:// URLs are supported."))?;

            let code = std::fs::read_to_string(&path).map_err(JsErrorBox::from_err)?;

            let should_transpile = self.options.transpile_suffix.iter().any(|suffix| {
                if let Some(extension) = path.extension() {
                    if extension == suffix.as_str() {
                        return true;
                    }
                }
                false
            });

            let code = if should_transpile {
                let transpiled =
                    transform_typescript(&code, &path.to_string_lossy()).map_err(|e| {
                        JsErrorBox::generic(format!(
                            "Failed to transpile module {}: {}",
                            module_specifier, e
                        ))
                    })?;

                if let Some(source_map) = transpiled.source_map {
                    self.source_maps
                        .write()
                        .insert(module_specifier.to_string(), source_map.into_bytes());
                }

                transpiled.code
            } else {
                code
            };

            Ok(ModuleSource::new(
                deno_core::ModuleType::JavaScript,
                ModuleSourceCode::String(code.into()),
                module_specifier,
                None,
            ))
        })())
    }

    fn get_source_map(&self, specifier: &str) -> Option<Cow<'_, [u8]>> {
        self.source_maps
            .read()
            .get(specifier)
            .map(|v| v.clone().into())
    }
}

pub mod specifier;

use crate::module_loader::specifier::{ModuleSpecifier, ModuleType};
use ahash::HashMap;
use ahash::HashSet;
use deno_core::ModuleCodeBytes;
use deno_core::ModuleLoadOptions;
use deno_core::ModuleLoadReferrer;
use deno_core::ModuleLoadResponse;
use deno_core::ModuleLoader as DenoModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSourceCode;
use deno_core::ResolutionKind;
use deno_core::error::ModuleLoaderError;
use deno_core::resolve_import;
use deno_error::JsErrorBox;
use parking_lot::RwLock;
use std::borrow::Cow;
use std::fmt::Debug;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use tracing::trace_span;
use url::{ParseError, Url};

#[derive(thiserror::Error, Debug, deno_error::JsError)]
#[class(generic)]
pub enum ModuleLoadError {
    #[error("Get an url parse error:{0}")]
    UrlParseError(#[from] ParseError),
    #[error("Other error: {0:?}")]
    Other(#[from] eyre::Report),
}

pub type SourceMapStore =
    Arc<RwLock<std::collections::HashMap<String, Vec<u8>, ahash::RandomState>>>;

pub type LoadedSourceSets = Arc<RwLock<std::collections::HashSet<PathBuf, ahash::RandomState>>>;

pub type AsyncLoadHook = Box<
    dyn Fn(PathBuf) -> Pin<Box<dyn Future<Output = Result<ModuleSource, ModuleLoaderError>>>>
        + 'static,
>;

pub struct LoaderOptions {
    pub read_module: HashMap<ModuleSpecifier, String>,
    pub load_hook: AsyncLoadHook,
}

impl Debug for LoaderOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoaderOptions")
            .field("read_module", &self.read_module)
            .finish()
    }
}

impl Default for LoaderOptions {
    fn default() -> Self {
        Self {
            read_module: HashMap::with_hasher(ahash::RandomState::new()),
            load_hook: Box::new(|path| {
                let callback = async move || {
                    let text = tokio::fs::read(&path).await.unwrap();

                    Ok(ModuleSource::new(
                        deno_core::ModuleType::JavaScript,
                        ModuleSourceCode::Bytes(ModuleCodeBytes::Boxed(text.into())),
                        &Url::from_file_path(path).unwrap(),
                        None,
                    ))
                };
                let callback = callback();

                return Box::pin(callback);
            }),
        }
    }
}

impl LoaderOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub struct ModuleLoader {
    _read_module: im::HashMap<ModuleSpecifier, String, ahash::RandomState>,
    source_maps: SourceMapStore,
    _loaded_source_sets: LoadedSourceSets,
}

impl ModuleLoader {
    pub fn new(options: LoaderOptions) -> Self {
        let mut read_module = im::HashMap::with_hasher(ahash::RandomState::new());
        read_module.extend(options.read_module);

        Self {
            _read_module: read_module,
            source_maps: Arc::new(RwLock::new(std::collections::HashMap::with_hasher(
                ahash::RandomState::new(),
            ))),
            _loaded_source_sets: Arc::new(RwLock::new(HashSet::with_hasher(
                ahash::RandomState::new(),
            ))),
        }
    }
}

impl DenoModuleLoader for ModuleLoader {
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
                    let resolved =
                        resolve_import(&specifier.url.to_string(), &referrer.url.to_string())
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

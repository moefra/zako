use crate::builtin;
use crate::engine::State;
use crate::module_loader::ModuleLoadError::NotSupported;
use crate::module_specifier::ModuleSpecifier;
use crate::path::NeutralPath;
use crate::sandbox::{Sandbox, SandboxError};
use crate::transformer::transform_typescript;
use ahash::AHashMap;
use eyre::Result;
use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};
use thiserror::Error;
use tracing::error;
use tracing::trace_span;
use v8::callback_scope;
use v8::script_compiler::Source;
use v8::{Data, FixedArray, Local, PinScope, Promise, PromiseResolver, ScriptOrigin, Value};

pub static NEEDED_TRANSFORMED_FILE_EXTENSION: &[&'static str] = &["ts", "mts"];

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Options {
    pub enable_imports: bool,
}

#[derive(Debug)]
pub struct ModuleLoader {
    options: Options,
    sandbox: Arc<Sandbox>,
    module_map: RefCell<AHashMap<v8::Global<v8::Module>, ModuleSpecifier>>,
    module_cache: RefCell<AHashMap<ModuleSpecifier, v8::Global<v8::Module>>>,
    dependencies: RefCell<AHashMap<ModuleSpecifier, Vec<ModuleSpecifier>>>,
    import_map: RefCell<AHashMap<String, ModuleSpecifier>>,
}

#[derive(Error, Debug)]
pub enum ModuleLoadError {
    #[error("Not found module: `{specifier:?}` referer `{referer:?}`")]
    NotFound {
        referer: ModuleSpecifier,
        specifier: ModuleSpecifier,
    },
    #[error(
        "Can not load memory module or load esm file from memory/builtin/import-map esm:`{specifier}` referer `{referer}`"
    )]
    NotSupported {
        referer: ModuleSpecifier,
        specifier: ModuleSpecifier,
    },
    #[error("Invalid module path: {0}")]
    PathError(#[from] crate::path::PathError),
    #[error("Invalid io operation: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Sandbox error: {0}")]
    SandboxError(#[from] SandboxError),
    #[error(
        "Failed to allocate V8 object. It may because v8 run out of memory or the object is too large:{0}"
    )]
    V8ObjectAllocationError(&'static str),
    #[error("Failed to compile module: {0}")]
    V8CompileError(ModuleSpecifier),
    #[error("Failed to instantiate and evaluate module: {0:?}")]
    V8InstaniateAndEvaluateError(ModuleSpecifier),
    #[error(
        "Failed to set synthetic module export `{0}`(Note: it may because of duplicated export or unknown export)"
    )]
    V8SyntheticModuleBuildingError(&'static str),
    #[error("Failed to find resolved module specifier: {0:?}")]
    UnknownModuleSpecifier(ModuleSpecifier),
    #[error("Failed to find builtin module: {0}")]
    UnknownBuiltinModuleSpecifier(String),
    #[error("Failed to transform typescript module `{0:?}`:{1}")]
    FailedToTransformTypescript(ModuleSpecifier, String),
}

impl ModuleLoader {
    pub fn new(sandbox: Arc<Sandbox>, options: Options) -> Self {
        Self {
            options,
            sandbox,
            module_map: RefCell::from(AHashMap::new()),
            module_cache: RefCell::from(AHashMap::new()),
            dependencies: RefCell::from(AHashMap::new()),
            import_map: RefCell::from(AHashMap::new()),
        }
    }

    /// Resolve path
    fn resolve_module_specifier(
        self: &Self,
        referrer: &ModuleSpecifier,
        specifier: &ModuleSpecifier,
    ) -> Result<ModuleSpecifier, ModuleLoadError> {
        match specifier.clone() {
            ModuleSpecifier::Builtin(builtin) => Ok(ModuleSpecifier::Builtin(builtin)),
            ModuleSpecifier::Memory(_memory) => Err(NotSupported {
                referer: referrer.clone(),
                specifier: specifier.clone(),
            }),
            ModuleSpecifier::ImportMap(import_map) => {
                if let Some(mapped) = self.import_map.borrow().get(&import_map) {
                    Ok(mapped.clone())
                } else {
                    Err(ModuleLoadError::NotFound {
                        referer: referrer.clone(),
                        specifier: specifier.clone(),
                    })
                }
            }
            ModuleSpecifier::File(target) => {
                if let ModuleSpecifier::File(referrer_path) = referrer {
                    let target = NeutralPath::new(target.to_string_lossy())?;

                    let target = self.sandbox.get_path_safe(referrer_path, &target)?;

                    let target = ModuleSpecifier::File(target);

                    self.dependencies
                        .borrow_mut()
                        .entry(referrer.clone())
                        .or_default()
                        .push(target.clone());

                    Ok(target)
                } else {
                    Err(NotSupported {
                        referer: referrer.clone(),
                        specifier: specifier.clone(),
                    })
                }
            }
        }
    }

    /// Get and compile module
    ///
    /// We process file modules and builtin modules here.
    ///
    /// Import-map and memory module has been resolved in `resolve` method.
    pub fn resolve_module<'s, 'i>(
        self: &Self,
        scope: &PinScope<'s, 'i>,
        specifier: &ModuleSpecifier,
    ) -> Result<Local<'s, v8::Module>, ModuleLoadError> {
        let module = if let Some(global_mod) = self.module_cache.borrow().get(specifier) {
            Local::new(scope, global_mod)
        } else {
            let origin = ScriptOrigin::new(
                scope,
                v8::String::new(scope, specifier.clone().to_string().as_str())
                    .ok_or(ModuleLoadError::V8ObjectAllocationError(
                        "v8::String::new(scope,specifier.to_string())",
                    ))?
                    .into(),
                0,
                0,
                false,
                0,
                None,
                false,
                false,
                true,
                None,
            );

            let module = match specifier {
                ModuleSpecifier::Builtin(builtin_name) => {
                    if specifier.eq(&crate::builtin::js::RT) {
                        let v8_source = v8::String::new(scope, crate::builtin::js::RT_CODE).ok_or(
                            ModuleLoadError::V8ObjectAllocationError(
                                "v8::String::new(scope, &crate::builtin::js::RT_CODE)",
                            ),
                        )?;

                        let module = v8::script_compiler::compile_module(
                            scope,
                            &mut Source::new(v8_source, Some(&origin)),
                        )
                        .ok_or_else(|| ModuleLoadError::V8CompileError(specifier.clone()))?;

                        module
                    } else if specifier.eq(&crate::builtin::js::SYSCALL) {
                        // note: to modify syscall,see crate::builtin::js
                        v8::Module::create_synthetic_module(
                            scope,
                            v8::String::new(scope, specifier.to_string().as_ref()).ok_or(
                                ModuleLoadError::V8ObjectAllocationError(
                                    "v8::String::new(scope, &crate::builtin::js::SYSCALL)",
                                ),
                            )?,
                            builtin::js::get_exports(scope)?.as_slice(),
                            builtin::js::evalution_callback,
                        )
                    } else {
                        return Err(ModuleLoadError::UnknownBuiltinModuleSpecifier(
                            builtin_name.clone(),
                        ));
                    }
                }
                ModuleSpecifier::File(path_buf) => {
                    let mut source_code = std::fs::read_to_string(path_buf)?;

                    if {
                        let mut need_transform = false;
                        for need_transform_extension in NEEDED_TRANSFORMED_FILE_EXTENSION.iter() {
                            if let Some(extension) = path_buf.extension()
                                && extension.eq(std::ffi::OsStr::new(need_transform_extension))
                            {
                                need_transform = true;
                                break;
                            }
                        }
                        need_transform
                    } {
                        source_code = transform_typescript(
                            source_code.as_str(),
                            path_buf.to_string_lossy().to_string().as_str(),
                        )
                        .map_err(|err| {
                            ModuleLoadError::FailedToTransformTypescript(specifier.clone(), err)
                        })?;
                    }

                    let v8_source = v8::String::new(scope, &source_code).ok_or(
                        ModuleLoadError::V8ObjectAllocationError(
                            "v8::String::new(scope, &source_code)",
                        ),
                    )?;

                    let module = v8::script_compiler::compile_module(
                        scope,
                        &mut Source::new(v8_source, Some(&origin)),
                    )
                    .ok_or_else(|| ModuleLoadError::V8CompileError(specifier.clone()))?;

                    module
                }
                _ => return Err(ModuleLoadError::UnknownModuleSpecifier(specifier.clone())),
            };

            let global_mod = v8::Global::new(scope, module);

            self.module_cache
                .borrow_mut()
                .insert(specifier.clone(), global_mod.clone());
            self.module_map
                .borrow_mut()
                .insert(global_mod.clone(), specifier.clone());

            module
        };

        let module = Local::new(scope, module);

        Ok(module)
    }

    pub fn instantiate_and_evaluate_module<'s, 'i>(
        self: &Self,
        scope: &PinScope<'s, 'i>,
        module: &Local<v8::Module>,
    ) -> Option<Local<'s, v8::Value>> {
        if module.get_status() == v8::ModuleStatus::Uninstantiated {
            if !module.instantiate_module(scope, Self::resolve_module_hook)? {
                return None;
            }
        }

        if module.get_status() == v8::ModuleStatus::Instantiated {
            let result = module.evaluate(scope)?;

            return Some(result);
        }

        if module.get_status() == v8::ModuleStatus::Evaluated {
            return Some(module.get_module_namespace());
        }

        None
    }

    fn resolve_module_hook<'s, 'i>(
        context: v8::Local<'s, v8::Context>,
        specifier: v8::Local<'s, v8::String>,
        import_attributes: v8::Local<'s, v8::FixedArray>,
        referrer: v8::Local<'s, v8::Module>,
    ) -> Option<v8::Local<'s, v8::Module>> {
        callback_scope!(unsafe scope, context);

        let state = match scope.get_current_context().get_slot::<State>() {
            Some(state) => state,
            None => {
                error!("failed to get state from slot");
                return None;
            }
        };

        let referer = {
            let global_referrer = v8::Global::new(scope, referrer);
            match state
                .module_loader
                .module_map
                .borrow()
                .get(&global_referrer)
            {
                Some(module) => module.clone(),
                None => {
                    error!("failed to get loaded module from module map");
                    return None;
                }
            }
        };

        let specifier = specifier.to_rust_string_lossy(scope);
        let specifier = ModuleSpecifier::from(specifier);

        let resolved = match state
            .module_loader
            .resolve_module_specifier(&referer, &specifier)
        {
            Ok(resolved) => resolved,
            Err(err) => {
                error!("failed to resolve module specifier: {}", err);
                return None;
            }
        };

        match state.module_loader.resolve_module(scope, &resolved) {
            Ok(module) => Some(module),
            Err(err) => {
                error!("failed to resolve module: {}", err);
                None
            }
        }
    }

    fn load_module_async_hook<'s, 'i>(
        scope: &mut PinScope<'s, 'i>,
        host_defined_options: Local<'s, Data>,
        resource_name: Local<'s, Value>,
        specifier: Local<'s, v8::String>,
        import_attributes: Local<'s, FixedArray>,
    ) -> Option<Local<'s, Promise>> {
        let state = match scope.get_current_context().get_slot::<State>() {
            Some(state) => state,
            None => {
                error!("failed to get state from slot");
                return None;
            }
        };

        let referer = ModuleSpecifier::from(resource_name.to_rust_string_lossy(scope));
        let specifier = specifier.to_rust_string_lossy(scope);
        let specifier = ModuleSpecifier::from(specifier);

        let resolved = match state
            .module_loader
            .resolve_module_specifier(&referer, &specifier)
        {
            Ok(resolved) => resolved,
            Err(err) => {
                error!("failed to resolve module specifier: {}", err);
                return None;
            }
        };

        let module = match state.module_loader.resolve_module(scope, &resolved) {
            Ok(module) => module,
            Err(err) => {
                error!("failed to resolve module: {}", err);
                return None;
            }
        };

        let module = match state
            .module_loader
            .instantiate_and_evaluate_module(scope, &module)
        {
            Some(module) => module,
            None => {
                error!(
                    "failed to instantiate and evaluate module: specifier {}, request from {}",
                    specifier.to_string(),
                    resource_name.to_rust_string_lossy(scope)
                );
                return None;
            }
        };

        let resolver = PromiseResolver::new(scope).unwrap();

        match resolver.resolve(scope, module) {
            Some(_) => (),
            None => {
                error!("failed to resolve PromiseResolver");
                return None;
            }
        };

        Some(resolver.get_promise(scope))
    }

    pub fn execute_module<'s, 'i>(
        self: &Self,
        scope: &mut PinScope<'s, 'i>,
        module_specifier: impl AsRef<ModuleSpecifier>,
    ) -> Result<Local<'s, v8::Value>, ModuleLoadError> {
        let span = trace_span!(
            "execute module: {module_specifier}",
            module_specifier = module_specifier.as_ref().to_string()
        );
        let _span = span.enter();

        let module_specifier = module_specifier.as_ref();

        let module = self.resolve_module(scope, module_specifier)?;

        let module = self.instantiate_and_evaluate_module(scope, &module).ok_or(
            ModuleLoadError::V8InstaniateAndEvaluateError(module_specifier.clone()),
        )?;

        Ok(module)
    }

    pub fn apply(&self, isolate: &mut v8::OwnedIsolate) {
        isolate.set_host_import_module_dynamically_callback(Self::load_module_async_hook);
    }
}

use crate::{builtin, v8utils};
use crate::engine::{EngineError, State};
use crate::module_loader::ModuleLoadError::NotSupported;
use crate::module_specifier::ModuleSpecifier;
use crate::path::NeutralPath;
use crate::sandbox::{Sandbox, SandboxError};
use crate::transformer::transform_typescript;
use ahash::AHashMap;
use eyre::Result;
use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};
use std::cell::RefMut;
use thiserror::Error;
use tracing::{error, trace};
use tracing::trace_span;
use v8::{callback_scope, CallbackScope, Context, Global, HandleScope, Isolate, OwnedIsolate, PinnedRef, TryCatch};
use v8::script_compiler::Source;
use v8::{Data, FixedArray, Local, PinScope, Promise, PromiseResolver, ScriptOrigin, Value};
use crate::v8error::V8Error;
use crate::v8utils::with_try_catch;

pub static NEEDED_TRANSFORMED_FILE_EXTENSION: &[&'static str] = &["ts", "mts"];

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Options {
    pub enable_imports: bool,
}

#[derive(Debug)]
pub struct ModuleLoader {
    options: Options,
    sandbox: Arc<Sandbox>,
    module_to_specifier: RefCell<AHashMap<v8::Global<v8::Module>, ModuleSpecifier>>,
    specifier_to_module: RefCell<AHashMap<ModuleSpecifier, v8::Global<v8::Module>>>,
    loaded_modules: RefCell<AHashMap<ModuleSpecifier, Vec<ModuleSpecifier>>>,
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
    V8InstantiateAndEvaluateError(ModuleSpecifier),
    #[error("Failed to instantiate and evaluate module `{0:?}`:{1}")]
    V8InstantiateAndEvaluateErrorWithReason(ModuleSpecifier, String),
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
    #[error("Caught exception when execute module `{0:?}`:{1:?}")]
    CaughtException(ModuleSpecifier, V8Error),
    #[error("Module `{0:?}` execution rejected promise: {1:?}")]
    RejectedPromise(ModuleSpecifier, V8Error, Global<Promise>),
}

impl ModuleLoader {
    pub fn new(sandbox: Arc<Sandbox>, options: Options) -> Self {
        Self {
            options,
            sandbox,
            module_to_specifier: RefCell::from(AHashMap::new()),
            specifier_to_module: RefCell::from(AHashMap::new()),
            loaded_modules: RefCell::from(AHashMap::new()),
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
            ModuleSpecifier::Memory(_memory) => Err(ModuleLoadError::NotSupported {
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

                    self.loaded_modules
                        .borrow_mut()
                        .entry(referrer.clone())
                        .or_default()
                        .push(target.clone());

                    Ok(target)
                } else {
                    Err(ModuleLoadError::NotSupported {
                        referer: referrer.clone(),
                        specifier: specifier.clone(),
                    })
                }
            }
        }
    }

    fn get_builtin_source(specifier: &ModuleSpecifier) -> Option<&'static str> {
        if specifier.eq(&crate::builtin::js::RT) { return Some(builtin::js::RT_CODE); }
        if specifier.eq(&crate::builtin::js::GLOBAL) { return Some(builtin::js::GLOBAL_CODE); }
        if specifier.eq(&crate::builtin::js::CONSOLE) { return Some(builtin::js::CONSOLE_CODE); }
        if specifier.eq(&crate::builtin::js::SEMVER) { return Some(builtin::js::SEMVER_CODE); }
        if specifier.eq(&crate::builtin::js::CORE) { return Some(builtin::js::CORE_CODE); }
        if specifier.eq(&crate::builtin::js::PROJECT) { return Some(builtin::js::PROJECT_CODE); }
        None
    }

    /// Get and compile module
    ///
    /// We process file modules and builtin modules here.
    ///
    /// Import-map and memory module has been resolved in `resolve` method.
    fn load_and_compile_module<'s, 'i>(
        self: &Self,
        scope: &PinScope<'s, 'i>,
        specifier: &ModuleSpecifier,
    ) -> Result<Local<'s, v8::Module>, ModuleLoadError> {
        let span = trace_span!(
            "resolve module specific: {module_specifier}",
            module_specifier = specifier.to_string()
        );
        let _span = span.enter();

        let module = if let Some(global_mod) = self.specifier_to_module.borrow().get(specifier) {
            trace!("hit module cache for specifier: {}", specifier.to_string());
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
                    let code = Self::get_builtin_source(specifier);

                    if let Some(code) = code{
                        let v8_source = v8::String::new(scope,
                                                        code).ok_or(
                            ModuleLoadError::V8ObjectAllocationError(
                                "v8::String::new(scope, &BUILTIN_CODE)",
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

            self.specifier_to_module
                .borrow_mut()
                .insert(specifier.clone(), global_mod.clone());
            self.module_to_specifier
                .borrow_mut()
                .insert(global_mod.clone(), specifier.clone());

            module
        };

        let module = Local::new(scope, module);

        Ok(module)
    }

    pub fn execute_module(self: &Self,
                          scope: &mut PinnedRef<'_,HandleScope<'_>>,
                          context: Global<Context>,
                          module_specifier: &ModuleSpecifier)
                          -> Result<Result<Global<Value>,Global<Value>>, ModuleLoadError> {
        let mut isolate = &mut ***scope;

        // 如果模块抛出异常，这里直接返回 Err
        // 如果模块是同步的，返回 Global<Value>
        // 如果模块是 TLA (Top-Level Await)，返回 Global<Promise>
        let (value,promise) =
            match with_try_catch(
                &mut isolate,
                &context,
                |mut scope, context| {
                    let state = context
                        .get_slot::<State>()
                        .expect("State not found in context slot");

                    return match state.module_loader.resolve_module_and_evaluate(&mut scope, module_specifier) {
                        Ok(value) => {
                            if value.is_promise() {
                                let promise = v8::Local::<Promise>::try_from(value).unwrap();
                                Ok((
                                    None,
                                    Some(Global::new(&mut scope, promise))
                                ))
                            } else {
                                Ok((
                                    Some(Global::new(&mut scope, value)),
                                    None
                                ))
                            }
                        }
                        Err(e) => {
                            Err(e)
                        }
                    };
                }).map_err(|x| {
                    ModuleLoadError::CaughtException(
                        module_specifier.clone(),
                        x)
            })?{
                Ok(res)=> res?,
                Err(err)=> {
                    return Ok(Err(err));
                }
            };

        // 获取结果
        return if let Some(value) = value{
            Ok(Ok(value))
        }
        else if let Some(promise) = promise{
            match with_try_catch(
                &mut isolate,
                &context,
                |mut scope, _context| {
                    let result =
                        v8utils::run_event_loop_until_resolved(&mut scope, &context, &promise)
                            .map_err(|v8error| {
                                ModuleLoadError::CaughtException(
                                    module_specifier.clone(),
                                    v8error)
                            })?;

                    let promise = Local::new(&mut scope, promise);

                    match result{
                        Err(obj)=>{
                            let result = promise.result(&mut scope);
                            v8utils::check_try_catch(&mut scope, Option::from(module_specifier))
                                .map_err(|v8error| {
                                    ModuleLoadError::CaughtException(
                                        module_specifier.clone(),
                                        v8error)
                                })?;
                            Ok(Global::new(&mut scope, result))
                        },
                        Ok(obj)=>{
                            let obj = Local::new(scope, obj);
                            let promise = Global::new(&scope, promise);
                            Err(ModuleLoadError::RejectedPromise(
                                module_specifier.clone(),
                                v8utils::convert_rejected_promise_result_to_error(&mut scope, obj),
                                promise
                            ))
                        }
                    }
                })
                .map_err(|v8error| {
                    ModuleLoadError::CaughtException(
                        module_specifier.clone(),
                        v8error)
                })?{
                Ok(res)=> res?,
                Err(err)=> {
                     Ok(Err(err))
                }
            };
        }
        else{
            unreachable!("Either value or promise must be Some");
        }
    }

    fn instantiate_and_evaluate_module<'s, 'i>(
        self: &Self,
        scope: &PinScope<'s, 'i>,
        module: &Local<v8::Module>,
    ) -> Option<Local<'s, v8::Value>> {
        let module_key = v8::Global::new(scope,module);

        self.module_to_specifier.borrow_mut().get(&module_key).inspect(|ok| {
            trace!(
                "try instantiate and evaluate module `{module_specifier}`, status: {status:?}",
                module_specifier = ok.to_string(),
                status = module.get_status()
            )
        });

        if module.get_status() == v8::ModuleStatus::Uninstantiated {
            if !module.instantiate_module(scope, Self::resolve_module_hook)? {
                return None;
            }
        }

        if module.get_status() == v8::ModuleStatus::Instantiated {
            let result = module.evaluate(scope)?;

            if result.is_promise(){
                let promise = v8::Local::<v8::Promise>::try_from(result).unwrap();

                // promise.await
                return match promise.state() {
                    v8::PromiseState::Fulfilled => {
                        Some(module.get_module_namespace())
                    }
                    v8::PromiseState::Rejected => {
                        let result = promise.result(scope);
                        error!("module evaluation promise rejected: {}", result.to_rust_string_lossy(scope));

                        None
                    }
                    v8::PromiseState::Pending => {
                        Some(result) // 返回 Promise 给上层处理
                    }
                }
            }

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
                .module_to_specifier
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

        let span = trace_span!(
                "load module `{module_specifier}`, request from `{referer}`",
            referer = &referer.to_string(),
            module_specifier = &specifier,
        );
        let _span = span.enter();

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

        match state.module_loader.load_and_compile_module(scope, &resolved) {
            Ok(module) => Some(module),
            Err(err) => {
                error!("failed to resolve module: {}", err);
                None
            }
        }
    }

    fn load_module_async_hook<'s, 'i>(
        mut scope: &mut PinScope<'s, 'i>,
        _host_defined_options: Local<'s, Data>,
        resource_name: Local<'s, Value>,
        specifier: Local<'s, v8::String>,
        _import_attributes: Local<'s, FixedArray>,
    ) -> Option<Local<'s, Promise>> {
        let span = trace_span!(
          "load module `{module_specifier}` async, request from `{referer}`",
            referer = resource_name.to_rust_string_lossy(scope),
            module_specifier = specifier.to_rust_string_lossy(scope),
        );

        let _span = span.enter();

        let state = match scope.get_current_context().get_slot::<State>() {
            Some(state) => state,
            None => {
                error!("failed to get state from slot");
                return None;
            }
        };

        let context = Global::new(scope,scope.get_current_context());

        let module_specifier = ModuleSpecifier::from(
            specifier.to_rust_string_lossy(scope)
        );

        let result =
            match state.module_loader.execute_module(
                &mut scope,
                context,
                &module_specifier){
                Err(err) => {
                    error!("failed to execute module async: {}", err);

                    if let ModuleLoadError::RejectedPromise(_, _, promise) = err {
                        Local::new(scope, &promise)
                    } else {
                        let error_msg = format!("failed to load module `{}`: {}", module_specifier.to_string(), err);
                        let error = v8::String::new(scope, &error_msg).unwrap();

                        let resolver =
                            match PromiseResolver::new(scope){
                                None => {
                                    error!("failed to create PromiseResolver");
                                    return None;
                                },
                                Some(resolver) => resolver
                            };

                        resolver.reject(
                            scope,
                            v8::Exception::error(scope, error).into(),
                        );

                        return Some(resolver.get_promise(scope));
                    }
                },
                Ok(resolved)=> Local::new(scope, resolved)
            };

        let result = Local::new(scope, result);

        let resolver = PromiseResolver::new(scope).unwrap();

        match resolver.resolve(scope, result) {
            Some(_) => (),
            None => {
                error!("failed to resolve PromiseResolver");
                return None;
            }
        };

        Some(resolver.get_promise(scope))
    }

    /// If this return a promise, the caller need to await it.
    fn resolve_module_and_evaluate<'s>(
        self: &Self,
        scope: &mut PinScope<'s, '_>,
        module_specifier: impl AsRef<ModuleSpecifier>,
    ) -> Result<Local<'s,Value>, ModuleLoadError> {
        let span = trace_span!(
            "execute module: {module_specifier}",
            module_specifier = module_specifier.as_ref().to_string()
        );
        let _span = span.enter();

        let module_specifier = module_specifier.as_ref();

        let module = self.load_and_compile_module(&*scope, module_specifier)?;

        let module = self.instantiate_and_evaluate_module(&*scope, &module).ok_or(
            ModuleLoadError::V8InstantiateAndEvaluateError(module_specifier.clone()),
        )?;

        Ok(module)
    }

    pub fn apply(&self, isolate: &mut v8::OwnedIsolate) {
        isolate.set_host_import_module_dynamically_callback(Self::load_module_async_hook);
    }
}

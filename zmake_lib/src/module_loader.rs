use std::{cell::RefCell, rc::Rc};
use std::path::PathBuf;
use std::sync::Arc;
use crate::sandbox::{Sandbox, SandboxError};
use ahash::AHashMap;
use crate::path::NeutralPath;
use eyre::Result;
use thiserror::Error;
use v8::{Data, FixedArray, Global, Handle, Local, PinScope, Promise, PromiseResolver, ScriptOrigin, Value};
use v8::script_compiler::Source;
use crate::module_loader::ModuleLoadError::NotSupported;
use crate::module_specifier::{ModuleSpecifier, BUILTIN_MODULE_PREFIX};

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct Options {
    pub enable_imports: bool,
}

#[derive(Debug)]
pub struct ModuleLoader {
    options: Options,
    sandbox: Arc<Sandbox>,
    module_map:RefCell<AHashMap<v8::Global<v8::Module>,ModuleSpecifier>>,
    module_cache: RefCell<AHashMap<ModuleSpecifier, v8::Global<v8::Module>>>,
    dependencies: RefCell<AHashMap<ModuleSpecifier, Vec<ModuleSpecifier>>>,
    import_map: RefCell<AHashMap<String, ModuleSpecifier>>,
}

#[derive(Error, Debug)]
pub enum ModuleLoadError {
    #[error("Not found module: `{specifier:?}` imported from `{referer:?}`")]
    NotFound{
        referer: ModuleSpecifier,
        specifier: ModuleSpecifier,
    },
    #[error("Can not load memory module or load esm file from memory/builtin/import-map esm: `{specifier}` imported from `{referer}`")]
    NotSupported{
        referer: ModuleSpecifier,
        specifier: ModuleSpecifier,
    },
    #[error("Invalid module path: {0}")]
    PathError(#[from] crate::path::PathError),
    #[error("Sandbox error: {0}")]
    SandboxError(#[from] SandboxError),
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
    fn resolve(self:&Rc<Self>, specifier: &ModuleSpecifier, referrer: &ModuleSpecifier)
        -> Result<ModuleSpecifier,ModuleLoadError> {

        match specifier.clone() {
            ModuleSpecifier::Builtin(builtin) => {
                Ok(ModuleSpecifier::Builtin(builtin))
            },
            ModuleSpecifier::Memory(_memory) => {
                Err(NotSupported {
                    referer: referrer.clone(),
                    specifier: specifier.clone(),
                })
            },
            ModuleSpecifier::ImportMap(import_map) => {
                if let Some(mapped) = self.import_map.borrow().get(&import_map) {
                    Ok(mapped.clone())
                } else {
                    Err(ModuleLoadError::NotFound {
                        referer: referrer.clone(),
                        specifier: specifier.clone(),
                    })
                }
            },
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
                    Err(
                        NotSupported {
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
    pub fn load_module<'s,'i>(
        self:&Rc<Self>,
        scope: &PinScope<'s, 'i>,
        specifier: &ModuleSpecifier
    ) -> Option<(Local<'s, v8::Value>,Local<'s, v8::Module>)> {
        let module = if let Some(global_mod) = self.module_cache.borrow().get(specifier) {
            Local::new(scope, global_mod)
        }
        else {
            match specifier {
                ModuleSpecifier::Builtin(builtin_name) => {
                    todo!();

                    if builtin_name == "zmake:fs" {
                        let source_code = "export const fs = ...".to_string();

                        let v8_source = v8::String::new(scope, &source_code).unwrap();

                        let module = v8::script_compiler::compile_module(scope, &mut Source::new(v8_source, None)).unwrap();

                        let global_mod = v8::Global::new(scope, module);
                        self.module_cache.borrow_mut().insert(specifier.clone(), global_mod);

                        module
                    } else {
                        return None;
                    }
                },
                ModuleSpecifier::File(path_buf) => {
                    let source_code = std::fs::read_to_string(path_buf).ok()?;

                    let v8_source = v8::String::new(scope, &source_code).unwrap();

                    let origin = ScriptOrigin::new(scope,
                        v8::String::new(scope, path_buf.to_string_lossy().as_ref()).unwrap().into(),
                        0,
                        0,
                        false,
                        0,
                        None,
                        false,
                        false,
                        true,
                                                   None
                    );

                    let module = v8::script_compiler::compile_module(scope,
                                                                     &mut Source::new(v8_source, Some(&origin))).unwrap();

                    let global_mod = v8::Global::new(scope, module);

                    self.module_cache.borrow_mut().insert(specifier.clone(), global_mod.clone());
                    self.module_map.borrow_mut().insert(global_mod.clone(), specifier.clone());

                    module
                },
                _ => {
                    return None;
                }
            }
        };

        let module = Local::new(scope,module);

        loop{
            match module.get_status(){
                v8::ModuleStatus::Uninstantiated => {
                    module.instantiate_module(scope, |a,b,c,d|{
                        Self::load_module_hook(scope, a,b,c,d)
                    })?;
                },
                v8::ModuleStatus::Instantiating => {
                    std::thread::yield_now();
                },
                v8::ModuleStatus::Instantiated => {
                    return Some((module.evaluate(scope)?,module));
                },
                v8::ModuleStatus::Evaluating => {
                    std::thread::yield_now();
                },
                v8::ModuleStatus::Evaluated => {
                    return Some((module.get_module_namespace(),module));
                },
                v8::ModuleStatus::Errored=> {
                    return None;
                }
            }
        }
    }

    fn load_module_hook<'s,'i>(
              scope:&PinScope<'s, 'i>,
              context: v8::Local<'s, v8::Context>,
              specifier: v8::Local<'s, v8::String>,
              import_attributes: v8::Local<'s, v8::FixedArray>,
              referrer: v8::Local<'s, v8::Module>,
           ) -> Option<v8::Local<'s, v8::Module>>
    {
        let loader = context.get_slot::<ModuleLoader>()?;

        let referer = {
            let global_referrer = v8::Global::new(scope, referrer);
            loader.module_map.borrow().get(&global_referrer)?.clone()
        };

        let specifier = specifier.to_rust_string_lossy(scope);
        let specifier = ModuleSpecifier::from(specifier);

        let resolved = loader.resolve(&referer,&specifier).ok()?;

        let module = loader.load_module(scope, &resolved)?;

        Some(module.1)
    }

    fn load_module_async_hook<'s,'i>(
        scope:&mut PinScope<'s, 'i>,
        host_defined_options:Local<'s, Data>,
        resource_name:Local<'s, Value>,
        specifier:Local<'s, v8::String>,
        import_attributes:Local<'s, FixedArray>,
    ) -> Option<Local<'s, Promise>>
    {
        let loader = scope.get_slot::<Rc<ModuleLoader>>()?;

        let referer = ModuleSpecifier::from(resource_name.to_rust_string_lossy(scope));
        let specifier = specifier.to_rust_string_lossy(scope);
        let specifier = ModuleSpecifier::from(specifier);

        let resolved = loader.resolve(&referer,&specifier).ok()?;

        let module = loader.load_module(scope, &resolved)?;

        let resolver = PromiseResolver::new(scope).unwrap();

        resolver.resolve(scope, module.0.into())?;

        Some(resolver.get_promise(scope))
    }

    pub fn apply(self,isolate:&mut v8::OwnedIsolate) -> Rc<Self>{
        let rc = Rc::new(self);

        // we set slot when we load module with context
        isolate.set_slot(rc.clone());

        isolate.set_host_import_module_dynamically_callback(Self::load_module_async_hook);

        rc
    }
}

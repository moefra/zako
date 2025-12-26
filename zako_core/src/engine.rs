use crate::module_loader::{LoaderOptions, ModuleLoader, specifier::ModuleSpecifier};
use crate::v8_platform::get_set_platform_or_default;
use crate::v8error::{ExecutionResult, V8Error};
use crate::{builtin, consts, v8error, v8utils};
use deno_core::error::CoreError;
use deno_core::serde_v8;
use deno_core::{JsRuntime, RuntimeOptions, v8};
use serde_json;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use thiserror::Error;
use tracing::trace_span;
use v8::{Local, PinScope};

#[derive(Debug)]
pub struct EngineOptions {
    pub tokio_handle: tokio::runtime::Handle,
    pub context_type: crate::consts::V8ContextType,
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Get an core error")]
    CoreError(#[from] CoreError),

    #[error("Get a V8 error:{0:?}")]
    V8Error(v8error::V8Error),
}

/// An engine,that should be used in one thread.
pub struct Engine {
    options: EngineOptions,
    runtime: Rc<RefCell<JsRuntime>>,
}

impl Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("options", &self.options)
            .finish()
    }
}

impl Engine {
    pub fn new(options: EngineOptions) -> Result<Self, EngineError> {
        JsRuntime::init_platform(Some(get_set_platform_or_default()), false);

        let loader = Rc::new(ModuleLoader::new(LoaderOptions {
            ..Default::default()
        }));

        // TODO: add extensions for different context types
        // Issue URL: https://github.com/moefra/zako/issues/19
        match options.context_type {
            consts::V8ContextType::Project => {}
            consts::V8ContextType::Build => {}
            consts::V8ContextType::Rule => {}
            consts::V8ContextType::Toolchain => {}
        }

        let runtime = JsRuntime::try_new(RuntimeOptions {
            module_loader: Some(loader.clone()),
            extensions: vec![
                builtin::extension::rt::zako_rt::init(),
                builtin::extension::syscall::zako_syscall::init(),
                builtin::extension::global::zako_global::init(),
                builtin::extension::semver::zako_semver::init(),
                builtin::extension::core::zako_core::init(),
                builtin::extension::console::zako_console::init(),
            ],
            ..Default::default()
        })?;

        let engine = Engine {
            options,
            runtime: Rc::new(RefCell::new(runtime)),
        };

        //engine.execute_build_script()?;

        Ok(engine)
    }

    pub fn execute_module(
        &mut self,
        module_specifier: &ModuleSpecifier,
        source_code: Option<String>,
    ) -> Result<deno_core::v8::Global<deno_core::v8::Object>, EngineError> {
        let _span =
            trace_span!("Engine::execute_module", module_specifier = %module_specifier).entered();
        let js_runtime = self.runtime.clone();

        let future = async move {
            let mut js_runtime = js_runtime.borrow_mut();
            let module_id = if let Some(source_code) = source_code {
                js_runtime
                    .load_main_es_module_from_code(&module_specifier.url, source_code)
                    .await?
            } else {
                js_runtime
                    .load_main_es_module(&module_specifier.url)
                    .await?
            };
            let result = js_runtime.mod_evaluate(module_id);
            js_runtime.run_event_loop(Default::default()).await?;
            result.await?;
            Ok::<deno_core::v8::Global<deno_core::v8::Object>, EngineError>(
                js_runtime.get_module_namespace(module_id)?,
            )
        };
        Ok(self.options.tokio_handle.block_on(future)?)
    }

    pub fn execute_module_with_json(
        &mut self,
        module_specifier: &ModuleSpecifier,
        source_code: Option<String>,
        json_input: serde_json::Value,
    ) -> Result<deno_core::v8::Global<deno_core::v8::Object>, EngineError> {
        let _span = trace_span!(
            "Engine::execute_module_with_json",
            module_specifier = %module_specifier
        )
        .entered();
        let js_runtime = self.runtime.clone();

        let future = async move {
            let mut js_runtime = js_runtime.borrow_mut();

            // Set globalThis.executionContext
            {
                let context = js_runtime.main_context();
                let isolate = js_runtime.v8_isolate();
                let handle = v8::HandleScope::<v8::Context>::new(isolate);
                let scope = std::pin::pin!(handle);
                let mut scope = scope.init();

                let context_local = v8::Local::new(&mut scope, context);
                let mut scope = v8::ContextScope::new(&mut scope, context_local);

                let global = context_local.global(&mut scope);
                let key = v8::String::new(&mut scope, "executionContext").unwrap();
                let value = serde_v8::to_v8(&mut scope, json_input).map_err(|e| {
                    EngineError::V8Error(V8Error {
                        message: format!("Failed to convert JSON to V8: {}", e),
                        ..Default::default()
                    })
                })?;
                global.set(&mut scope, key.into(), value);
            }

            let module_id = if let Some(source_code) = source_code {
                js_runtime
                    .load_main_es_module_from_code(&module_specifier.url, source_code)
                    .await?
            } else {
                js_runtime
                    .load_main_es_module(&module_specifier.url)
                    .await?
            };
            let result = js_runtime.mod_evaluate(module_id);
            js_runtime.run_event_loop(Default::default()).await?;
            result.await?;
            Ok::<deno_core::v8::Global<deno_core::v8::Object>, EngineError>(
                js_runtime.get_module_namespace(module_id)?,
            )
        };
        Ok(self.options.tokio_handle.block_on(future)?)
    }

    pub fn execute_module_and_then<F, R>(
        self: &mut Self,
        module_specifier: &ModuleSpecifier,
        then: F,
    ) -> Result<R, EngineError>
    where
        F: for<'s> FnOnce(
            &mut PinScope<'s, '_>,
            v8::Local<v8::Context>,
            v8::Local<'s, v8::Object>,
        ) -> R,
    {
        let _span =
            trace_span!("Engine::execute_module_and_then", module_specifier = %module_specifier)
                .entered();
        let js_runtime = self.runtime.clone();

        let future = async move {
            let mut js_runtime = js_runtime.borrow_mut();
            let mod_id = js_runtime
                .load_main_es_module(&module_specifier.url)
                .await?;
            let result = js_runtime.mod_evaluate(mod_id);
            js_runtime.run_event_loop(Default::default()).await?;
            result.await?;

            let result = {
                let object = js_runtime.get_module_namespace(mod_id)?;

                let context = js_runtime.main_context();
                let isolate = js_runtime.v8_isolate();

                match v8utils::with_try_catch(isolate, &context, |mut scope, context| {
                    let object = Local::new(&scope, &object);
                    let result = then(&mut scope, context, object);
                    result
                }) {
                    Ok(result) => result,
                    Err(e) => return Err(EngineError::V8Error(e)),
                }
            };

            js_runtime.run_event_loop(Default::default()).await?;

            let context = js_runtime.main_context();
            let isolate = js_runtime.v8_isolate();

            return match result {
                ExecutionResult::Value(value) => Ok(value),
                ExecutionResult::Exception(exception) => Err(v8utils::with_context_scope(
                    isolate,
                    &context,
                    |mut scope: &mut v8::PinnedRef<'_, v8::HandleScope<'_>>, _context| {
                        let exception = Local::new(&mut scope, &exception);

                        let error = v8utils::convert_object_to_error(&mut scope, exception);

                        EngineError::V8Error(error)
                    },
                )),
            };
        };

        Ok(self.options.tokio_handle.block_on(future)?)
    }

    pub fn execute_module_with_json_and_then<F, R>(
        self: &mut Self,
        module_specifier: &ModuleSpecifier,
        source_code: Option<String>,
        json_input: serde_json::Value,
        then: F,
    ) -> Result<R, EngineError>
    where
        F: for<'s> FnOnce(
            &mut PinScope<'s, '_>,
            v8::Local<v8::Context>,
            v8::Local<'s, v8::Object>,
        ) -> R,
    {
        let _span = trace_span!(
            "Engine::execute_module_with_json_and_then",
            module_specifier = %module_specifier
        )
        .entered();

        let object_global =
            self.execute_module_with_json(module_specifier, source_code, json_input)?;

        let js_runtime_rc = self.runtime.clone();

        let result = {
            let mut js_runtime = js_runtime_rc.borrow_mut();
            let context_global = js_runtime.main_context();
            let isolate = js_runtime.v8_isolate();

            match v8utils::with_try_catch(isolate, &context_global, |mut scope, context| {
                let object = Local::new(&scope, &object_global);
                let result = then(&mut scope, context, object);
                Ok(result)
            }) {
                Ok(ExecutionResult::Value(Ok(result))) => ExecutionResult::Value(result),
                Ok(ExecutionResult::Value(Err(e))) => return Err(EngineError::V8Error(e)),
                Ok(ExecutionResult::Exception(exception)) => ExecutionResult::Exception(exception),
                Err(e) => return Err(EngineError::V8Error(e)),
            }
        };

        let future = async move {
            let mut js_runtime = js_runtime_rc.borrow_mut();
            js_runtime.run_event_loop(Default::default()).await?;

            let context = js_runtime.main_context();
            let isolate = js_runtime.v8_isolate();

            return match result {
                ExecutionResult::Value(value) => Ok(value),
                ExecutionResult::Exception(exception) => Err(v8utils::with_context_scope(
                    isolate,
                    &context,
                    |mut scope: &mut v8::PinnedRef<'_, v8::HandleScope<'_>>, _context| {
                        let exception = Local::new(&mut scope, &exception);

                        let error = v8utils::convert_object_to_error(&mut scope, exception);

                        EngineError::V8Error(error)
                    },
                )),
            };
        };

        Ok(self.options.tokio_handle.block_on(future)?)
    }

    pub fn get_runtime(self: &Self) -> Rc<RefCell<JsRuntime>> {
        self.runtime.clone()
    }
}

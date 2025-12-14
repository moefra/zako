use crate::v8_platform::get_set_platform_or_default;
use crate::v8error::{ExecutionResult, V8Error};
use crate::zako_module_loader::{LoaderOptions, ModuleSpecifier, ZakoModuleLoader};
use crate::{builtin, v8error, v8utils};
use deno_core::error::CoreError;
use deno_core::v8::{HandleScope, PinnedRef, TryCatch};
use deno_core::{JsRuntime, RuntimeOptions, v8};
use prost::Message;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use thiserror::Error;
use tracing::trace_span;
use tracing_attributes::instrument;
use v8::{Context, Global, Isolate, Local, Object, PinScope, PromiseResolver, Value};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EngineMode {
    Project,
    Rule,
    Build,
}

#[derive(Debug)]
pub struct EngineOptions {
    pub tokio_handle: tokio::runtime::Handle,
    pub mode: EngineMode,
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Get an core erro")]
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

        let loader = Rc::new(ZakoModuleLoader::new(LoaderOptions {
            ..Default::default()
        }));

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
                    |mut scope: &mut v8::PinnedRef<'_, v8::HandleScope<'_>>, context| {
                        let exception = Local::new(&mut scope, &exception);

                        let error = v8utils::convert_object_to_error(&mut scope, exception);

                        EngineError::V8Error(error)
                    },
                )),
                ExecutionResult::Promise(promise) => {
                    unreachable!(
                        "promise should have been resolved after JsRuntime::run_event_loop"
                    )
                }
            };
        };

        Ok(self.options.tokio_handle.block_on(future)?)
    }

    pub fn get_runtime(self: &Self) -> Rc<RefCell<JsRuntime>> {
        self.runtime.clone()
    }
}

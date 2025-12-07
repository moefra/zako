use crate::builtin;
use crate::platform::get_set_platform_or_default;
use crate::sandbox::{Sandbox, SandboxRef};
use crate::zako_module_loader::{LoaderOptions, ModuleSpecifier, ZakoModuleLoader};
use deno_core::error::CoreError;
use deno_core::{JsRuntime, RuntimeOptions, v8};
use prost::Message;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use thiserror::Error;
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
    #[error("Get an core error:{0}")]
    CoreError(#[from] CoreError),
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

    #[instrument]
    pub fn execute_module(
        self: &mut Self,
        module_specifier: &ModuleSpecifier,
    ) -> Result<Global<Object>, EngineError> {
        let js_runtime = self.runtime.clone();

        let future = async move {
            let mut js_runtime = js_runtime.borrow_mut();
            let mod_id = js_runtime
                .load_main_es_module(&module_specifier.url)
                .await?;
            let result = js_runtime.mod_evaluate(mod_id);
            js_runtime.run_event_loop(Default::default()).await?;
            result.await?;
            js_runtime.get_module_namespace(mod_id)
        };

        Ok(self.options.tokio_handle.block_on(future)?)
    }
}

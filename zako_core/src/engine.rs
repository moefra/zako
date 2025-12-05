use crate::module_loader::{ModuleLoadError, ModuleLoader, Options};
use crate::module_specifier::ModuleSpecifier;
use crate::platform::get_initialized_or_default;
use crate::sandbox::Sandbox;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use thiserror::Error;
use std::sync::mpsc::Sender;
use prost::Message;
use v8::{Context, Global, Isolate, Local, Object, PinScope, PromiseResolver, Value};
use crate::future::PendingOp;
use crate::v8error::V8Error;
use crate::v8utils;
use crate::v8utils::with_try_catch;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EngineMode {
    Project,
    Rule,
}

#[derive(Debug)]
pub struct EngineOptions {
    pub tokio_handle: tokio::runtime::Handle,
    pub mode: EngineMode,
}

#[derive(Debug)]
pub struct State {
    pub mode: EngineMode,
    pub tokio_handle: tokio::runtime::Handle,
    pub module_loader: ModuleLoader,
    pub async_tx: Sender<PendingOp>,
}

#[derive(Error, Debug)]
pub enum EngineError{
    #[error("Caught exception when execute module `{0:?}`:{1:?}")]
    CaughtException(ModuleSpecifier, V8Error),
    #[error("Get error when execute module `{0:?}`:{1}")]
    ExecutionError(ModuleSpecifier,String),
    #[error("Get error from module loader:{0}")]
    ModuleLoadError(#[from] ModuleLoadError),
}

#[derive(Debug)]
pub struct Engine {
    sandbox: Arc<Sandbox>,
    isolate: RefCell<v8::OwnedIsolate>,
    context: Global<v8::Context>,
    receiver: RefCell<std::sync::mpsc::Receiver<PendingOp>>,
    state: Rc<State>,
}

impl Engine {
    pub fn new(sandbox: Arc<Sandbox>, options: EngineOptions) -> eyre::Result<Self> {
        let _ = get_initialized_or_default();

        let channel = std::sync::mpsc::channel();
        let mut isolate = v8::Isolate::new(v8::CreateParams::default());

        let loader = ModuleLoader::new(
            sandbox.clone(),
            Options {
                enable_imports: true,
            },
        );

        loader.apply(&mut isolate);

        let (context,state) = {
            let handle_scope = std::pin::pin!(v8::HandleScope::new(&mut isolate));
            let mut handle_scope = handle_scope.init();

            let context = v8::Context::new(&mut handle_scope, Default::default());
            let scope = &mut v8::ContextScope::new(&mut handle_scope, context);

            let state = Rc::from(State {
                mode: options.mode,
                tokio_handle: options.tokio_handle.clone(),
                module_loader: loader,
                async_tx: channel.0
            });

            context.set_slot::<State>(state.clone());

            (Global::new(scope, context),state)
        };

        let mut engine = Engine {
            sandbox,
            isolate: RefCell::from(isolate),
            context,
            receiver: RefCell::from(channel.1),
            state
        };

        _ = engine.execute_module(&crate::builtin::js::RT)?;
        _ = engine.execute_module(&crate::builtin::js::GLOBAL)?;

        Ok(engine)
    }

    pub fn execute_module(self: &mut Self, module_specifier: &ModuleSpecifier)
                               -> Result<Global<Value>, EngineError>{
        let state = self.state.clone();

        let result = with_try_catch(
            self.isolate.get_mut(),
                &self.context,
            |scope,context|{
                let context = Global::new(scope, context);
                state.module_loader.execute_module(
                    scope,
                    context,
                    module_specifier
                )
            }
        ).map_err(|err|{
            EngineError::CaughtException(
                module_specifier.clone(),
                err
            )
        })?;

        Ok(result?)
    }

    pub fn get_sandbox(&self) -> Arc<Sandbox> {
        self.sandbox.clone()
    }
}

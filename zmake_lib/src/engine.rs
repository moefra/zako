use crate::module_loader::{ModuleLoader, Options};
use crate::platform::get_initialized_or_default;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::{Arc, LazyLock};
use tracing::{trace, trace_span};
use v8::{Context, Global};
use crate::sandbox::Sandbox;

#[derive(Debug)]
pub struct EngineOptions {
    pub tokio_handle: tokio::runtime::Handle,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineOptions {
    pub fn new() -> Self {
        EngineOptions {
            tokio_handle: tokio::runtime::Handle::try_current().unwrap_or_else(|_| {
                panic!("Engine must be initialized within a Tokio runtime context");
            }),
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    isolate: v8::OwnedIsolate,
    context: Global<v8::Context>,
    tokio_handle: tokio::runtime::Handle,
}

impl Engine {
    pub fn new_resolve_engine(sandbox:Arc<Sandbox>,options: EngineOptions) -> eyre::Result<Self> {
        let _ = get_initialized_or_default();

        let mut isolate = v8::Isolate::new(v8::CreateParams::default());
        let context = {
            let handle_scope = std::pin::pin!(v8::HandleScope::new(&mut isolate));
            let mut handle_scope = handle_scope.init();

            let context = v8::Context::new(&mut handle_scope, Default::default());
            let scope = &mut v8::ContextScope::new(&mut handle_scope, context);

            Global::new(scope, context)
        };

        let loader = ModuleLoader::new(sandbox,Options {
            enable_imports: true,
        });

        let _loader = loader.apply(&mut isolate);

        Ok(Engine {
            isolate,
            context,
            tokio_handle: options.tokio_handle,
        })
    }
}

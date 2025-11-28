use crate::module_loader::{ModuleLoader, Options};
use crate::module_specifier::ModuleSpecifier;
use crate::platform::get_initialized_or_default;
use crate::sandbox::Sandbox;
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::{Arc, LazyLock};
use tracing::{trace, trace_span};
use v8::{Context, Global, Local};

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
    isolate: RefCell<v8::OwnedIsolate>,
    context: Global<v8::Context>,
    tokio_handle: tokio::runtime::Handle,
    module_loader: Rc<ModuleLoader>,
}

impl Engine {
    pub fn new_resolve_engine(
        sandbox: Arc<Sandbox>,
        options: EngineOptions,
    ) -> eyre::Result<Rc<Self>> {
        let _ = get_initialized_or_default();

        let mut isolate = v8::Isolate::new(v8::CreateParams::default());

        let context = {
            let handle_scope = std::pin::pin!(v8::HandleScope::new(&mut isolate));
            let mut handle_scope = handle_scope.init();

            let context = v8::Context::new(&mut handle_scope, Default::default());
            let scope = &mut v8::ContextScope::new(&mut handle_scope, context);

            Global::new(scope, context)
        };

        let loader = ModuleLoader::new(
            sandbox,
            Options {
                enable_imports: true,
            },
        );

        let module_loader = loader.apply(&mut isolate);

        let engine = Rc::from(Engine {
            isolate: RefCell::from(isolate),
            context,
            tokio_handle: options.tokio_handle,
            module_loader,
        });

        Rc::get_mut(&mut engine.clone())
            .unwrap()
            .isolate
            .get_mut()
            .set_slot(engine.clone());

        engine.clone().execute_module(&crate::builtin::js::RT);

        Ok(engine)
    }

    pub fn execute_module(mut self: Rc<Self>, module: &ModuleSpecifier) {
        let context = self.context.clone();
        let module_loader = self.module_loader.clone();
        let scope = std::pin::pin!(v8::HandleScope::new(
            Rc::get_mut(&mut self).unwrap().isolate.get_mut()
        ));
        let mut scope = scope.init();
        let context = Local::new(&scope, context);
        let mut scope = &mut v8::ContextScope::new(&mut scope, context);
        module_loader.execute_module(&mut scope, module).unwrap();
    }
}

use crate::module_loader::{ModuleLoader, Options};
use crate::module_specifier::ModuleSpecifier;
use crate::platform::get_initialized_or_default;
use crate::sandbox::Sandbox;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use v8::{Global, Local};

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
}

#[derive(Debug)]
pub struct Engine {
    isolate: RefCell<v8::OwnedIsolate>,
    context: Global<v8::Context>,
}

impl Engine {
    pub fn new(sandbox: Arc<Sandbox>, options: EngineOptions) -> eyre::Result<Self> {
        let _ = get_initialized_or_default();

        let mut isolate = v8::Isolate::new(v8::CreateParams::default());

        let loader = ModuleLoader::new(
            sandbox,
            Options {
                enable_imports: true,
            },
        );

        loader.apply(&mut isolate);

        let context = {
            let handle_scope = std::pin::pin!(v8::HandleScope::new(&mut isolate));
            let mut handle_scope = handle_scope.init();

            let context = v8::Context::new(&mut handle_scope, Default::default());
            let scope = &mut v8::ContextScope::new(&mut handle_scope, context);

            let state = State {
                mode: options.mode,
                tokio_handle: options.tokio_handle.clone(),
                module_loader: loader,
            };

            context.set_slot::<State>(Rc::from(state));

            Global::new(scope, context)
        };

        let engine = Engine {
            isolate: RefCell::from(isolate),
            context,
        };

        engine.execute_module(&crate::builtin::js::RT);

        Ok(engine)
    }

    pub fn execute_module(self: &Self, module: &ModuleSpecifier) {
        let context = self.context.clone();
        let mut isoalte = self.isolate.borrow_mut();
        let scope = std::pin::pin!(v8::HandleScope::new(&mut *isoalte));
        let mut scope = scope.init();
        let context = Local::new(&scope, context);

        let state = context.get_slot::<State>().unwrap();

        let mut scope = &mut v8::ContextScope::new(&mut scope, context);

        state
            .module_loader
            .execute_module(&mut scope, module)
            .unwrap();
    }
}

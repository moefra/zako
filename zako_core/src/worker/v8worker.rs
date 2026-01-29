use crate::{
    builtin::extension::{context::ContextInformation, package::PackageInformation},
    engine::{Engine, EngineError, EngineOptions},
    global_state::GlobalState,
    module_loader::{LoaderOptions, specifier::ModuleSpecifier},
    package::Package,
    v8context::{V8ContextInput, V8ContextOutput},
    v8utils,
    worker::WorkerBehavior,
};
use ::eyre::eyre;
use ::std::collections::HashMap;
use ::tracing::trace_span;
use deno_core::{Extension, serde_v8, snapshot};
use eyre::{Context, OptionExt};
use rkyv::Archive;
use serde_json;
use std::{fmt::Debug, rc::Rc, sync::Arc};
use tokio::runtime::{Handle, Runtime};
use tracing::{instrument, trace};
use zako_cancel::CancelToken;

/// This is the input of V8 Worker.
#[derive(Debug)]
pub struct V8WorkerInput {
    /// The file that will be imported and executed.
    pub specifier: String,

    // If the module is a typescript module, use this channel to request the transformer to transform it to javascript.
    pub request_channel: flume::Sender<crate::worker::protocol::V8ImportRequest>,

    /// The cached bytecode of the file.
    pub cached_bytecode: Option<Vec<u8>>,

    /// The context type of the script file.
    ///
    /// It provided script arguments and decide the type of return value.
    pub context_type: V8ContextInput,
}

/// Output from V8 Worker
#[derive(Debug)]
pub struct V8WorkerOutput {
    pub return_value: V8ContextOutput,
}

#[derive(Debug, thiserror::Error)]
pub enum V8WorkerError {
    #[error("Get an js engine error: {0}")]
    EngineError(#[from] EngineError),
    #[error("Get an serde_v8 error: {0}")]
    SerdeError(#[from] serde_v8::Error),
    #[error("Other error: {0}")]
    Other(#[from] eyre::Report),
}

/// A worker that executes JavaScript using V8 (via deno_core)
#[derive(Clone)]
pub struct V8Worker;

impl std::fmt::Debug for V8Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("V8Worker").finish()
    }
}

/// State for V8 Worker, holding the JsRuntime and a Tokio Runtime for async execution
pub struct V8State {
    handle: Handle,
    env: Arc<GlobalState>,
}

impl Debug for V8State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("V8State")
            .field("tokio runtime handle", &self.handle)
            .finish()
    }
}

impl V8Worker {
    #[cfg(not(feature = "v8snapshot"))]
    pub fn package_snapshot() -> eyre::Result<Box<[u8]>> {
        let context = crate::builtin::extension::context::ContextInformation {
            name: crate::builtin::extension::context::ContextName::Package,
        };

        let context = Rc::new(context);

        let context = crate::builtin::extension::context::zako_context::init(context);

        let package: PackageInformation = Default::default();

        let package_rc = Rc::new(package);

        let package = crate::builtin::extension::package::zako_package::init(package_rc.clone());

        let extensions = vec![context, package];

        Self::snapshot(extensions)
    }

    #[cfg(not(feature = "v8snapshot"))]
    fn snapshot(extensions: Vec<Extension>) -> eyre::Result<Box<[u8]>> {
        let (import_channel, _) = flume::unbounded();

        let runtime = tokio::runtime::Builder::new_current_thread().build()?;

        let engine = Engine::snapshot(
            EngineOptions {
                tokio_handle: runtime.handle().clone(),
                extensions,
                snapshot: None,
            },
            LoaderOptions {
                read_module: ahash::HashMap::default(),
                import_channel,
            },
        )?;

        Ok(engine.snapshot())
    }

    pub fn package(
        state: &mut V8State,
        input: V8WorkerInput,
        _cancel_token: CancelToken,
    ) -> Result<V8WorkerOutput, V8WorkerError> {
        let _span = trace_span!("v8 execution", input = ?input, mode="package").entered();

        let package = match input.context_type {
            V8ContextInput::Package { package } => package,
            _ => todo!(),
        };

        let context = crate::builtin::extension::context::ContextInformation {
            name: crate::builtin::extension::context::ContextName::Package,
        };

        let context = Rc::new(context);

        let context = crate::builtin::extension::context::zako_context::init(context);

        let config = package
            .resolved_config
            .resolve(state.env.interner())
            .wrap_err("failed to resolve ResolvedConfiguration to Configuration")?;

        let package = crate::builtin::extension::package::PackageInformation::new(package, config)
            .wrap_err("failed to create package information object for v8 script execution")?;

        let package_rc = Rc::new(package);

        let package = crate::builtin::extension::package::zako_package::init(package_rc.clone());

        let extensions = vec![context, package];

        if crate::v8snapshot::PACKAGE_SNAPSHOT.is_none() {
            trace!("use no snapshot")
        } else {
            trace!("use snapshot")
        }

        let mut engine = Engine::new(
            EngineOptions {
                tokio_handle: state.handle.clone(),
                extensions,
                snapshot: crate::v8snapshot::PACKAGE_SNAPSHOT,
            },
            LoaderOptions {
                read_module: ahash::HashMap::default(),
                import_channel: input.request_channel,
            },
        )?;

        let specifier = url::Url::from_file_path(&input.specifier)
            .map_err(|_| eyre!("failed to parse the {:?} into file url", input.specifier))?;

        _ = engine.execute_module(
            &ModuleSpecifier {
                url: specifier,
                module_type: crate::module_loader::specifier::ModuleType::File,
            },
            None,
        )?;

        drop(engine);

        let package: PackageInformation = match Rc::try_unwrap(package_rc) {
            Ok(package) => package,
            Err(package_rc) => (*package_rc).clone(),
        };
        let package = package.get_package();
        let return_value = V8ContextOutput::Package { package };

        Ok(V8WorkerOutput { return_value })
    }
}

impl WorkerBehavior for V8Worker {
    type Context = GlobalState;
    type Input = V8WorkerInput;
    type Output = Result<V8WorkerOutput, V8WorkerError>;
    type State = V8State;

    fn init(ctx: &Arc<Self::Context>) -> Self::State {
        V8State {
            handle: ctx.handle().clone(),
            env: ctx.clone(),
        }
    }

    #[instrument]
    fn process(
        state: &mut Self::State,
        input: Self::Input,
        cancel_token: CancelToken,
    ) -> Self::Output {
        match &input.context_type {
            &V8ContextInput::Package { .. } => Self::package(state, input, cancel_token),
            _ => todo!(),
        }
    }
}

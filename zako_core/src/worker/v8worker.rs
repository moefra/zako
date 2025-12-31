use crate::{
    engine::{Engine, EngineError, EngineOptions},
    global_state::GlobalState,
    module_loader::{LoaderOptions, specifier::ModuleSpecifier},
    v8context::{V8ContextInput, V8ContextOutput},
    v8utils,
    worker::WorkerBehavior,
};
use ::eyre::eyre;
use ::std::collections::HashMap;
use ::tracing::trace_span;
use deno_core::serde_v8;
use serde_json;
use std::{fmt::Debug, sync::Arc};
use tokio::runtime::Handle;
use tracing::instrument;
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
#[derive(Debug, Clone)]
pub struct V8Worker;

/// State for V8 Worker, holding the JsRuntime and a Tokio Runtime for async execution
pub struct V8State {
    handle: Handle,
}

impl Debug for V8State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("V8State")
            .field("tokio runtime handle", &self.handle)
            .finish()
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
        }
    }

    #[instrument]
    fn process(
        state: &mut Self::State,
        input: Self::Input,
        _cancel_token: CancelToken,
    ) -> Self::Output {
        let _span = trace_span!("v8 execution", input = ?input).entered();

        let mut engine = Engine::new(
            EngineOptions {
                tokio_handle: state.handle.clone(),
                extensions: vec![],
            },
            LoaderOptions {
                read_module: ahash::HashMap::default(),
                import_channel: input.request_channel,
            },
        )?;

        let runtime = engine.get_runtime();
        let mut runtime = runtime.borrow_mut();
        let context = runtime.main_context();
        let mut isolate = runtime.v8_isolate();
        v8utils::with_try_catch(&mut isolate, &context, |scope, context| {
            let global = context.global(scope);

            global.get(scope, key, value)?;

            Ok(())
        });

        let specifier = url::Url::from_file_path(&input.specifier)
            .map_err(|_| eyre!("failed to parse the {:?} into file url", input.specifier))?;

        let result = engine.execute_module_and_then(
            &ModuleSpecifier {
                url: specifier,
                module_type: crate::module_loader::specifier::ModuleType::File,
            },
            |scope, _context, object| {
                let rust_value: V8ContextOutput =
                    serde_v8::from_v8(scope, object.into()).map_err(V8WorkerError::SerdeError)?;
                Ok(V8WorkerOutput {
                    return_value: rust_value,
                })
            },
        )?;

        result
    }
}

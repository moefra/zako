use crate::{
    context::BuildContext,
    engine::{Engine, EngineError, EngineOptions},
    module_loader::specifier::ModuleSpecifier,
    worker::WorkerBehavior,
};
use deno_core::serde_v8;
use parking_lot::Mutex;
use std::{fmt::Debug, pin::Pin, sync::Arc};
use tokio::runtime::Handle;
use tracing::instrument;
use zako_cancel::CancelToken;

/// This is the input of V8 Worker.
#[derive(Debug)]
pub struct V8WorkerInput {
    /// The specifier of module that will be executed.
    pub specifier: ModuleSpecifier,

    /// The source code of the module that will be executed.
    ///
    /// It should not be typescript, use [crate::compute::transform_typescript] to transform it to javascript.
    pub source_code: String,

    // If the module is a typescript module, use this channel to request the transformer to transform it to javascript.
    pub request_channel: flume::Sender<crate::worker::protocol::V8Request>,

    /// 5. (可选) 预编译的字节码 (Optimization)
    /// 如果 Engine 缓存里有 V8 Bytecode，传进去可以跳过 Parsing
    pub cached_bytecode: Option<Vec<u8>>,

    /// 6. 上下文类型
    /// 决定引擎的权限和能力
    pub context_type: crate::consts::V8ContextType,
}

/// Output from V8 Worker
#[derive(Debug)]
pub struct V8WorkerOutput {
    pub result: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum V8WorkerError {
    #[error("Get an js engine error: {0}")]
    EngineError(#[from] EngineError),
    #[error("Get an serde_v8 error: {0}")]
    SerdeError(#[from] serde_v8::Error),
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
    type Context = BuildContext;
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
        let mut engine = Engine::new(EngineOptions {
            tokio_handle: state.handle.clone(),
            context_type: input.context_type,
        })?;

        engine.execute_module_and_then(&input.specifier, |scope, context, object| {
            let rust_value: serde_json::Value =
                serde_v8::from_v8(scope, object.into()).map_err(V8WorkerError::SerdeError)?;
            Ok(V8WorkerOutput { result: rust_value })
        })?
    }
}

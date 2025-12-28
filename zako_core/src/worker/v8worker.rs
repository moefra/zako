use crate::{
    engine::{Engine, EngineError, EngineOptions},
    global_state::GlobalState,
    module_loader::specifier::ModuleSpecifier,
    v8context::V8ContextInput,
    worker::WorkerBehavior,
};
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
    pub context_type: V8ContextInput,

    /// 7. (可选) JSON 输入
    pub json_input: Option<serde_json::Value>,
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

        let mut engine = Engine::new(EngineOptions {
            tokio_handle: state.handle.clone(),
            extensions: vec![],
        })?;

        let json_input = input.json_input.unwrap_or(serde_json::Value::Null);

        engine.execute_module_with_json_and_then(
            &input.specifier,
            Some(input.source_code),
            json_input,
            |scope, _context, object| {
                let rust_value: serde_json::Value =
                    serde_v8::from_v8(scope, object.into()).map_err(V8WorkerError::SerdeError)?;
                Ok(V8WorkerOutput { result: rust_value })
            },
        )?
    }
}

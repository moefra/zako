use crate::worker::WorkerBehavior;
use deno_core::{v8, JsRuntime, RuntimeOptions};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use zako_cancel::CancelToken;

/// Input for V8 Worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V8WorkerInput {
    /// The JavaScript code to execute
    pub script: String,
    /// Arguments to pass to the script (accessible via `args` global if configured)
    pub args: Vec<String>,
}

/// Output from V8 Worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V8WorkerOutput {
    /// The result of the execution as a string
    pub result: Option<String>,
    /// Any error message if execution failed
    pub error: Option<String>,
    /// Time taken to execute the script
    pub execution_time: Duration,
}

/// A worker that executes JavaScript using V8 (via deno_core)
pub struct V8Worker;

/// State for V8 Worker, holding the JsRuntime and a Tokio Runtime for async execution
pub struct V8State {
    js_runtime: JsRuntime,
    tokio_runtime: tokio::runtime::Runtime,
}

impl WorkerBehavior for V8Worker {
    type Input = V8WorkerInput;
    type Output = V8WorkerOutput;
    type State = V8State;

    fn init() -> Self::State {
        // Create a single-threaded tokio runtime for the worker thread
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime for V8 worker");

        // Initialize JsRuntime
        let js_runtime = JsRuntime::new(RuntimeOptions::default());

        V8State {
            js_runtime,
            tokio_runtime,
        }
    }

    fn process(
        state: &mut Self::State,
        input: Self::Input,
        _cancel_token: CancelToken,
    ) -> Self::Output {
        let start = Instant::now();
        let V8State {
            js_runtime,
            tokio_runtime,
        } = state;

        // Execute the script within the tokio runtime to handle async operations
        let result: Result<String, String> = tokio_runtime.block_on(async {
            // Execute the script
            // Note: In a real environment, you might want to wrap this in a try/catch block within JS
            // or use a specific module loader. Here we execute it as a simple script.
            let result_global = match js_runtime.execute_script("<anon>", input.script.into()) {
                Ok(g) => g,
                Err(e) => return Err(e.to_string()),
            };

            // Resolve the value (awaits promises if the result is a promise)
            let resolved_global = match js_runtime.resolve_value(result_global).await {
                Ok(g) => g,
                Err(e) => return Err(e.to_string()),
            };

            // Convert result to string
            let scope = &mut js_runtime.handle_scope();
            let local = v8::Local::new(scope, resolved_global);
            Ok(local.to_rust_string_lossy(scope))
        });

        match result {
            Ok(s) => V8WorkerOutput {
                result: Some(s),
                error: None,
                execution_time: start.elapsed(),
            },
            Err(e) => V8WorkerOutput {
                result: None,
                error: Some(e),
                execution_time: start.elapsed(),
            },
        }
    }

    fn clean(_state: Self::State) {
        // Resources (JsRuntime, Runtime) are dropped automatically
    }
}

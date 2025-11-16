use std::cell::Cell;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::LazyLock;
use deno_core::{JsRuntime, RuntimeOptions};
use tracing::{trace, trace_span};

pub struct Engine{
    pub runtime:JsRuntime,
}

impl Engine {
    pub fn new() -> anyhow::Result<Self>{
        let runtime = JsRuntime::try_new(
            RuntimeOptions{
                ..Default::default()
            }
        )?;

        Ok(Engine {
            runtime
        })
    }

    pub fn execute_and_to_json(&mut self, code:&str, source_name:&str) -> std::string::String {
        
    }
}

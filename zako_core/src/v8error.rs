//! The means a V8 error that can not continue.
//!
//! For some exception that can continue,do not use this,and use Value as an exception to broadcast the error in js.
//!
//! It was usually used as:
//! ```rust
//! Result<ExecutionResult,V8Error>.
//! //-----^result---------^fatal error that can not continue
//! ```
use crate::zako_module_loader::ModuleSpecifier;

#[derive(Clone, Debug, Default)]
pub struct V8Error {
    pub module_specifier: Option<ModuleSpecifier>,
    pub message: String,
    pub stack: Option<String>,
    pub exception: Option<String>,
    pub addition_message: Option<String>,
}

use deno_core::v8::{Global, Promise, Value};
use strum::Display;

/// 代表 JS 代码执行后的结果状态（前提是 V8 引擎本身没有崩溃）
#[derive(Debug)]
pub enum ExecutionResult<T> {
    /// JS 代码正常返回 (return value)
    Value(T),
    /// JS 代码抛出了异常 (throw error)
    Exception(Global<Value>),
    /// 需要支持 TLA
    Promise(Global<Promise>),
}

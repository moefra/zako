use v8::{Global, Promise, Value};

/// 代表 JS 代码执行后的结果状态（前提是 V8 引擎本身没有崩溃）
#[derive(Debug)]
pub enum ExecutionResult<T> {
    /// JS 代码正常返回 (return value)
    Value(T),
    /// JS 代码抛出了异常 (throw error)
    Exception(Global<Value>),
    // 需要支持 TLA
    Promise(Global<Promise>)
}

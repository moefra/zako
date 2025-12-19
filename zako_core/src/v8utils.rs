use crate::module_loader::specifier::ModuleSpecifier;
use crate::v8error::{ExecutionResult, V8Error};
use deno_core::anyhow::Context;
use deno_core::v8::{self, PinScope};
use deno_core::v8::{
    Global, HandleScope, Isolate, Local, OwnedIsolate, PinnedRef, Platform, Value,
};
use std::borrow::BorrowMut;
use std::cell::{RefCell, RefMut};
use std::ops::DerefMut;
use std::time::Duration;

pub fn check_try_catch<'s>(
    mut scope: &mut PinnedRef<v8::TryCatch<'s, '_, HandleScope>>,
    module_specifier: Option<&ModuleSpecifier>,
) -> Result<ExecutionResult<()>, V8Error> {
    if scope.has_caught() {
        if scope.can_continue() {
            return Ok(ExecutionResult::Exception(Global::new(
                scope,
                scope.exception().unwrap(),
            )));
        }

        let exception = scope.exception().unwrap();
        let msg = scope.message().unwrap();

        let stack = scope
            .stack_trace()
            .map(|s| s.to_rust_string_lossy(&mut scope))
            .unwrap_or_else(|| "no stack trace".to_string());

        return Err(V8Error {
            module_specifier: module_specifier.map(|x| x.clone()),
            exception: Option::from(exception.to_rust_string_lossy(&mut scope)),
            message: msg.get(&mut scope).to_rust_string_lossy(&mut scope),
            stack: stack.into(),
            ..Default::default()
        });
    }
    Ok(ExecutionResult::Value(()))
}

pub fn run_event_loop_until_resolved(
    scope: &mut PinnedRef<HandleScope>, // 正常传入 Scope
    context: &v8::Global<v8::Context>,
    promise_global: &v8::Global<v8::Promise>,
) -> Result<Result<v8::Global<v8::Value>, v8::Global<v8::Value>>, V8Error> {
    // 进入 Context 上下文
    let context_local = v8::Local::new(scope, context);
    let mut scope = v8::ContextScope::new(scope, context_local); // 现在 scope 是 ContextScope

    loop {
        let promise = v8::Local::new(&mut scope, promise_global);

        match promise.state() {
            v8::PromiseState::Fulfilled => {
                let result = promise.result(&mut scope);
                return Ok(Ok(v8::Global::new(&mut scope, result)));
            }
            v8::PromiseState::Rejected => {
                let result = promise.result(&mut scope);
                return Ok(Err(v8::Global::new(&mut scope, result)));
            }
            v8::PromiseState::Pending => {
                // 1. 先运行所有的微任务 (Promise.then 等)
                scope.perform_microtask_checkpoint();

                // 2. 再次检查状态，如果微任务执行完 Promise 变了，直接下一轮循环处理结果
                if promise.state() != v8::PromiseState::Pending {
                    continue;
                }

                // 3. 关键：驱动平台消息循环 (处理 setTimeout, GC 任务等)
                // WaitForOnlyOneTask: 阻塞等待直到有一个任务到来。这避免了 CPU 100% 空转。
                // 如果你有后台任务，这里会有效地挂起线程，直到有事可做。
                let platform_pumped = Platform::pump_message_loop(
                    &v8::V8::get_current_platform(),
                    &mut **scope,
                    true,
                );

                // 如果 pump 返回 false，说明没有更多的任务可以跑了，但 Promise 还是 Pending
                // 这通常意味着死锁（Promise 永远无法被解决），应该报错退出
                if !platform_pumped && promise.state() == v8::PromiseState::Pending {
                    return Err(V8Error {
                        message: "Event loop terminated but promise is still pending (Deadlock)"
                            .into(),
                        ..Default::default()
                    });
                }
            }
        }
    }
}

/// 这是一个辅助函数，封装了所有样板代码
///
/// F: 你的业务逻辑闭包
pub fn with_context_scope<F, R>(
    isolate: &mut v8::Isolate,
    context_global: &v8::Global<v8::Context>,
    f: F,
) -> R
where
    F: for<'pin, 'i> FnOnce(&mut v8::PinScope<'pin, 'i>, v8::Local<'pin, v8::Context>) -> R,
{
    let handle = HandleScope::<v8::Context>::new(isolate);
    let scope = std::pin::pin!(handle);
    let mut scope = scope.init();
    let context = v8::Local::new(&mut scope, context_global);
    let mut scope = v8::ContextScope::new(&mut scope, context);
    f(&mut scope, context)
}

/// 这是一个辅助函数，封装了所有样板代码
///
/// F: 你的业务逻辑闭包
///
/// 带 TryCatch 的版本
pub fn with_try_catch<F, R>(
    isolate: &mut Isolate,
    context_global: &Global<v8::Context>,
    f: F,
) -> Result<ExecutionResult<R>, V8Error>
where
    F: for<'s, 'obj> FnOnce(
        &mut PinnedRef<v8::TryCatch<'s, '_, HandleScope<'_>>>,
        v8::Local<'s, v8::Context>,
    ) -> R,
{
    with_context_scope(isolate, context_global, |scope, context| {
        let try_catch = std::pin::pin!(v8::TryCatch::new(scope));
        let mut try_catch = try_catch.init();

        let result = f(&mut try_catch, context);

        let err = check_try_catch(&mut try_catch, None);

        match err {
            Err(v8error) => Err(v8error),
            Ok(value) => match value {
                ExecutionResult::Value(()) => Ok(ExecutionResult::Value(result)),
                ExecutionResult::Exception(exception) => Ok(ExecutionResult::Exception(
                    Global::new(&mut try_catch, exception),
                )),
            },
        }
    })
}

pub fn get_stack_trace_string<'s, 'i>(
    scope: &mut v8::PinScope<'s, 'i>,
    exception: v8::Local<'s, v8::Value>,
) -> Option<String> {
    if !exception.is_object() {
        return None;
    }
    let object = exception.to_object(scope)?;

    let stack_key = v8::String::new(scope, "stack")?.into();

    let stack_value = object.get(scope, stack_key)?;

    if stack_value.is_null_or_undefined() {
        return None;
    }

    Some(stack_value.to_rust_string_lossy(scope))
}

pub fn convert_rejected_promise_to_error<'s, 'i>(
    scope: &mut v8::PinScope<'s, 'i>,
    promise: v8::Local<'s, v8::Promise>,
) -> V8Error {
    let exception = promise.result(scope);

    convert_object_to_error(scope, exception)
}

pub fn convert_object_to_error<'s, 'i>(
    scope: &mut v8::PinScope<'s, 'i>,
    exception: Local<'s, v8::Value>,
) -> V8Error {
    let error_msg = exception.to_rust_string_lossy(scope);

    V8Error {
        message: error_msg,
        stack: get_stack_trace_string(scope, exception),
        ..Default::default()
    }
}

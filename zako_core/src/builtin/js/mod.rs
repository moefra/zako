use tokio::sync::watch::error;
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::field::debug;
use v8::PinScope;
use crate::{make_builtin_js, module_loader::ModuleLoadError, module_specifier::ModuleSpecifier};
use crate::engine::State;

pub static RT_CODE: &'static str = include_str!(concat!(
    std::env!("CARGO_MANIFEST_DIR"),
    "/../dist/rt/rt.mjs"
));

pub static GLOBAL_CODE: &'static str = include_str!(concat!(
    std::env!("CARGO_MANIFEST_DIR"),
    "/../dist/global/global.mjs"
));

pub static CORE_CODE: &'static str = include_str!(concat!(
std::env!("CARGO_MANIFEST_DIR"),
"/../dist/core/core.mjs"
));

pub static SEMVER_CODE: &'static str = include_str!(concat!(
std::env!("CARGO_MANIFEST_DIR"),
"/../dist/semver/index.mjs"
));

pub static CONSOLE_CODE: &'static str = include_str!(concat!(
std::env!("CARGO_MANIFEST_DIR"),
"/../dist/console/console.mjs"
));

pub static PROJECT_CODE: &'static str = include_str!(concat!(
std::env!("CARGO_MANIFEST_DIR"),
"/../dist/project/project.mjs"
));

#[::static_init::dynamic(lazy)]
pub static RT: ModuleSpecifier = ModuleSpecifier::Builtin("rt".to_string());

#[::static_init::dynamic(lazy)]
pub static CONSOLE: ModuleSpecifier = ModuleSpecifier::Builtin("console".to_string());

#[::static_init::dynamic(lazy)]
pub static CORE: ModuleSpecifier = ModuleSpecifier::Builtin("core".to_string());

#[::static_init::dynamic(lazy)]
pub static PROJECT: ModuleSpecifier = ModuleSpecifier::Builtin("project".to_string());

#[::static_init::dynamic(lazy)]
pub static SEMVER: ModuleSpecifier = ModuleSpecifier::Builtin("semver".to_string());

#[::static_init::dynamic(lazy)]
pub static GLOBAL: ModuleSpecifier = ModuleSpecifier::Builtin("global".to_string());

#[::static_init::dynamic(lazy)]
pub static SYSCALL: ModuleSpecifier = ModuleSpecifier::Builtin("syscall".to_string());

/*
 *  To modify the name of method,remeber to modify it in js file too.
 */
make_builtin_js!(
    syscalls:{
        log
    }
    accessors:
    {
        version
    }
);

pub fn log<'s, 'i>(
    scope: &mut ::v8::PinScope<'s, 'i>,
    args: ::v8::FunctionCallbackArguments<'s>,
    mut return_value: ::v8::ReturnValue<'s, v8::Value>,
) {
    let level = args
        .get(0)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);
    let message = args
        .get(1)
        .to_string(scope)
        .unwrap()
        .to_rust_string_lossy(scope);

    match level.as_ref() {
        "trace" => {
            trace!("FROM SCRIPT {}", message);
        }
        "debug" => {
            debug!("FROM SCRIPT {}", message);
        }
        "info" => {
            info!("FROM SCRIPT {}", message);
        }
        "warn" => {
            warn!("FROM SCRIPT {}", message);
        }
        "error" => {
            error!("FROM SCRIPT {}", message);
        }
        _ => {
            error!("UNKNOWN LOG LEVEL FROM SCRIPT {}", message);
        }
    }

    return_value.set_undefined();
}

pub fn version<'s, 'i>(
    scope: &mut ::v8::PinScope<'s, 'i>,
) -> Result<v8::Local<'s, v8::Value>, ModuleLoadError> {
    Ok(v8::String::new(scope, env!("CARGO_PKG_VERSION"))
        .ok_or_else(|| {
            crate::module_loader::ModuleLoadError::V8ObjectAllocationError(
                "failed to create string",
            )
        })?
        .into())
}

/// TODO: Make this more safe
pub struct ThreadSafeGlobal<T>(pub v8::Global<T>);

unsafe impl<T> Send for ThreadSafeGlobal<T> {}
unsafe impl<T> Sync for ThreadSafeGlobal<T> {}

impl<T> ThreadSafeGlobal<T> {
    pub fn new(g: v8::Global<T>) -> Self {
        Self(g)
    }

    pub unsafe fn into_inner(self) -> v8::Global<T> {
        self.0
    }
}

fn delay<'s, 'i>(
    scope: &mut ::v8::PinScope<'s, 'i>,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let ms = args.get(0).to_integer(scope).unwrap().value();

    // 1. 创建 Resolver
    let resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = resolver.get_promise(scope);

    // 2. 返回 Promise 给 JS
    retval.set(promise.into());

    // 3. 准备跨线程数据

    let state = match scope.get_current_context().get_slot::<State>() {
        Some(state) => state,
        None => {
            error!("failed to get state from slot");
            return;
        }
    };

    let tx = state.async_tx.clone();

    let global_resolver = ThreadSafeGlobal::new(v8::Global::new(scope, resolver));

    // 4. 启动 Tokio 任务
    state.tokio_handle.spawn(async move {
        // 模拟耗时操作
        tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;

        tx.send(Box::new(move |scope:&mut PinScope| {
            let global_resolver = unsafe { global_resolver.into_inner() };
            let local_resolver = v8::Local::new(scope, global_resolver);
            let value = v8::undefined(scope);
            local_resolver.resolve(scope, value.into());
        })).unwrap();
    });
}
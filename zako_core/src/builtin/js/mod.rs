use tokio::sync::watch::error;
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::field::debug;

use crate::{make_builtin_js, module_loader::ModuleLoadError, module_specifier::ModuleSpecifier};

pub static RT_CODE: &'static str = include_str!(concat!(
    std::env!("CARGO_MANIFEST_DIR"),
    "/../dist/rt.js"
));

#[::static_init::dynamic(lazy)]
pub static RT: ModuleSpecifier = ModuleSpecifier::Builtin("rt".to_string());

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

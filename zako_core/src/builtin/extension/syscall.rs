use boxed_error::Boxed;
use deno_core::{FastString, ascii_str, op2};
use tracing::{debug, error, info, trace, warn};

#[derive(Debug, Boxed, deno_error::JsError)]
pub struct SyscallError(pub Box<SyscallErrorKind>);

#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum SyscallErrorKind {
    #[class(type)]
    #[error("Invalid log level: {0}")]
    InvalidLogLevel(String),
    #[class(generic)]
    #[error("String Interner error: {0}")]
    InternerError(#[from] ::zako_interner::InternerError),
}

deno_core::extension!(
    zako_syscall,
    deps = [zako_rt],
    ops = [syscall_core_version, syscall_core_log],
    esm_entry_point = "zako:syscall",
    esm = ["zako:syscall" = "../dist/builtins/syscall.js"],
    docs = "The extension that communicates between the script and the zako",
);

#[op2]
#[to_v8]
fn syscall_core_version() -> FastString {
    ascii_str!(env!("CARGO_PKG_VERSION")).into()
}

#[op2(fast)]
fn syscall_core_log(
    #[string] level: String,
    #[string] message: String,
) -> Result<(), SyscallError> {
    match level.as_ref() {
        "trace" => {
            trace!(" - {}", message);
        }
        "debug" => {
            debug!(" - {}", message);
        }
        "info" => {
            info!(" - {}", message);
        }
        "warn" => {
            warn!(" - {}", message);
        }
        "error" => {
            error!(" - {}", message);
        }
        _ => {
            return Err(SyscallError(Box::new(SyscallErrorKind::InvalidLogLevel(
                level,
            ))));
        }
    }
    Ok(())
}

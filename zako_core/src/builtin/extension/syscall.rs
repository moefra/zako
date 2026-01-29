use std::sync::OnceLock;

use boxed_error::Boxed;
use deno_core::{FastString, ascii_str, op2};
use tracing::{debug, error, info, trace, warn};

pub static ENABLE_PRINT: OnceLock<bool> = OnceLock::new();

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
fn syscall_core_version() -> FastString {
    ascii_str!(env!("CARGO_PKG_VERSION")).into()
}

#[op2(fast)]
fn syscall_core_log(
    #[string] level: String,
    #[string] message: String,
) -> Result<(), SyscallError> {
    let enable_print = *ENABLE_PRINT.get().unwrap_or(&true);
    match level.as_ref() {
        "trace" => {
            trace!(" - {}", message);
            if enable_print {
                println!("-- {} {}", "TRC", message);
            }
        }
        "debug" => {
            debug!(" - {}", message);
            if enable_print {
                println!("-- {} {}", "\x1b[0;96;49mDBG\x1b[0m", message);
            }
        }
        "info" => {
            info!(" - {}", message);
            if enable_print {
                println!("-- {} {}", "\x1b[0;97;49mINF\x1b[0m", message);
            }
        }
        "warn" => {
            warn!(" - {}", message);
            if enable_print {
                println!("-- {} {}", "\x1b[0;93;49mWRN\x1b[0m", message);
            }
        }
        "error" => {
            error!(" - {}", message);
            if enable_print {
                println!("-- {} {}", "\x1b[0;91;49mERR\x1b[0m", message);
            }
        }
        _ => {
            return Err(SyscallError(Box::new(SyscallErrorKind::InvalidLogLevel(
                level,
            ))));
        }
    }
    Ok(())
}

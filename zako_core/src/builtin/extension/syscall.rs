use deno_core::{FastString, ascii_str, op2};
use tracing::{debug, error, info, trace, warn};

deno_core::extension!(
    zako_syscall,
    deps = [zako_rt],
    ops = [syscall_version, syscall_log],
    esm_entry_point = "zako:syscall",
    esm = ["zako:syscall" = "builtins/syscall.js"],
    docs = "The extension that communicates between the script and the zako",
);

#[op2]
#[to_v8]
fn syscall_version() -> FastString {
    ascii_str!(env!("CARGO_PKG_VERSION")).into()
}

#[op2(fast)]
fn syscall_log(#[string] level: String, #[string] message: String) {
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
}

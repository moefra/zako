mod file;
mod glob;
mod parse_manifest;
mod resolve_label;
mod resolve_manifest_script;
mod resolve_package;
mod transpile_ts;

use ::camino::Utf8PathBuf;

use ::eyre::Context;
pub use file::file;
pub use glob::glob;
pub use parse_manifest::prase_manifest;
pub use resolve_label::resolve_label;
pub use resolve_manifest_script::resolve_manifest_script;
pub use resolve_package::resolve_package;
pub use transpile_ts::transpile_ts;

use crate::{
    compute::transpile_ts::transpile_ts_string,
    computer::ZakoComputeContext,
    v8context::{V8ContextInput, V8ContextOutput},
    worker::v8worker::V8WorkerInput,
};
use ::hone::{HoneResult, error::HoneError};

pub async fn execute_script<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    script: &str,
    input: V8ContextInput,
) -> HoneResult<V8ContextOutput> {
    let path = Utf8PathBuf::from(script);
    let path = path
        .canonicalize_utf8()
        .map_err(|e| HoneError::IOError(e, format!("{:?}", path)))?;

    let (tx, rx): (flume::Sender<crate::worker::protocol::V8ImportRequest>, _) = flume::unbounded();

    let worker = ctx.context().v8_workers_pool().submit(
        V8WorkerInput {
            specifier: path.to_string(),
            request_channel: tx,
            cached_bytecode: None,
            context_type: input,
        },
        ctx.cancel_token(),
    );

    while let Ok(request) = rx.recv_async().await {
        let request_script_path = path.join(request.specifier);

        let (_, handle) = file::read_text(ctx, request_script_path.as_str()).await?;

        let script =
            transpile_ts_string(ctx, request_script_path.to_string(), handle.content).await?;

        match request.resp.send(Ok(script.code)) {
            Err(_) => {
                break;
            }
            Ok(_) => (),
        }
    }

    drop(rx);

    Ok(worker
        .await
        .wrap_err("failed to submit v8 task")?
        .wrap_err("failed to execute javascript code")?
        .return_value)
}

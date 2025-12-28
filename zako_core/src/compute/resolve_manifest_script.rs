use ::eyre::{Context, OptionExt};
use ::hone::{HoneResult, error::HoneError, status::HashPair};
use ::smol_str::SmolStr;

use crate::{
    blob_range::BlobRange,
    computer::ZakoComputeContext,
    consts,
    intern::InternedAbsolutePath,
    module_loader::specifier::{ModuleSpecifier, ModuleType},
    node::{
        file::File,
        node_value::ZakoValue,
        resolve_manifest_script::{ResolveManifestScript, ResolveManifestScriptResult},
    },
    v8context::V8ContextInput,
    worker::v8worker::V8WorkerInput,
};

pub async fn resolve_manifest_script<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    key: &ResolveManifestScript,
) -> HoneResult<(HashPair, ResolveManifestScriptResult)> {
    let context = ctx.context();
    let interner = context.interner();
    let package = &key.package;
    let current = interner
        .resolve(context.project_root().interned)
        .wrap_err("failed to resolve project root")?;

    let path = format!("{}/{}", current, consts::PACKAGE_SCRIPT_FILE_NAME);
    let path_id = InternedAbsolutePath::new(&path, interner)
        .wrap_err("failed to resolve path")?
        .ok_or_eyre("failed to build absolute path")?;

    let cas_store = context.cas_store();

    let handle = ctx.request(File { path: path_id }.into()).await?;

    let handle = match &**handle.value() {
        ZakoValue::FileResult(file_result) => file_result,
        _ => {
            return Err(HoneError::UnexpectedError(
                "Expected FileResult".to_string(),
            ));
        }
    };

    let script = SmolStr::new(
        String::from_utf8(handle.content.read(cas_store, BlobRange::full()).await?).wrap_err_with(
            || format!("failed to read `{}` as valid utf-8 string", path.as_str()),
        )?,
    );

    let (tx, rx): (
        flume::Sender<crate::worker::protocol::V8TranspileRequest>,
        _,
    ) = flume::unbounded();

    let result = context.global_state().v8_workers_pool().submit(
        V8WorkerInput {
            specifier: ModuleSpecifier::new(
                url::Url::from_file_path(path.as_str())
                    .map_err(|_| eyre::eyre!("failed to build url from file path as specifier"))?,
                ModuleType::File,
            ),
            source_code: script.to_string(),
            request_channel: tx,
            cached_bytecode: None,
            context_type: V8ContextInput::Package {
                package: package.clone(),
            },
        },
        ctx.cancel_token(),
    );

    if let Ok(recv) = rx.recv_async().await {}

    let output_hash = handle.content.digest().blake3;

    Ok((
        HashPair {
            input_hash: input_hash.into(),
            output_hash: output_hash.into(),
        },
        ResolveLabelResult {
            target: crate::target::Target::Target(script),
        },
    ))
}

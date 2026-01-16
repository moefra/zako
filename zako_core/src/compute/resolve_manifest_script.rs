use ::eyre::{Context, OptionExt};
use ::hone::{HoneResult, error::HoneError, status::HashPair};
use ::smol_str::SmolStr;
use ::zako_digest::blake3::Blake3Hash;

use crate::{
    blob_range::BlobRange,
    compute::file,
    computer::ZakoComputeContext,
    consts,
    intern::InternedAbsolutePath,
    module_loader::specifier::{ModuleSpecifier, ModuleType},
    node::{
        file::File,
        node_value::ZakoValue,
        resolve_manifest_script::{ResolveManifestScript, ResolveManifestScriptResult},
    },
    v8context::{V8ContextInput, V8ContextOutput},
    worker::v8worker::V8WorkerInput,
};

pub async fn resolve_manifest_script<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    key: &ResolveManifestScript,
) -> HoneResult<(HashPair, ResolveManifestScriptResult)> {
    let context = ctx.context();
    let interner = context.interner();
    let package = key.package.clone();
    let current = interner
        .resolve(context.project_root())
        .wrap_err("failed to resolve project root")?;

    let path = format!("{}/{}", current, consts::PACKAGE_SCRIPT_FILE_NAME);

    let input_hash = package.get_blake3();

    let output = super::execute_script(ctx, &path, V8ContextInput::Package { package }).await?;

    let output = match output {
        V8ContextOutput::Package { package } => package,
        _ => {
            return Err(HoneError::UnexpectedError(
                "Unexpected v8 context output".to_string(),
            ));
        }
    };

    Ok((
        HashPair {
            input_hash: input_hash.into(),
            output_hash: output.get_blake3().into(),
        },
        ResolveManifestScriptResult { target: output },
    ))
}

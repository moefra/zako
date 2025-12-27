use ::eyre::Context;
use ::hone::{assert, debug_assert};
use ::smol_str::SmolStr;
use ::zako_digest::blake3_hash::Blake3Hash;
use ::zako_interner::InternerError;
use camino::Utf8Path;
use hone::{HoneResult, error::HoneError, status::HashPair};
use is_executable::is_executable;

use crate::{
    blob_handle::BlobHandle,
    blob_range::BlobRange,
    computer::ZakoComputeContext,
    consts, intern,
    node::{
        file::{File, FileResult},
        node_key::ZakoKey,
        node_value::ZakoValue,
        resolve_label::{ResolveLabel, ResolveLabelResult},
    },
};

pub async fn resolve_label<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    key: &ResolveLabel,
) -> HoneResult<(HashPair, ResolveLabelResult)> {
    let context: &crate::context::BuildContext = ctx.context();

    let interner = context.interner();

    let label = key
        .label
        .resolved(interner)
        .wrap_err("failed to resolve label")?;

    let input_hash = label.get_blake3();

    let package = &key.package;

    // check if it's configuration target
    if key.label.path.0
        == package
            .mount_config
            .map(|s| s.0)
            .unwrap_or(context.common_interneds().config_mount)
    {
        // construct configuration target
        todo!();
    }

    let path = interner
        .resolve(&key.label.path.0)
        .map_err(|err| HoneError::UnexpectedError(format!("Interner error: {}", err)))?;

    debug_assert!(
        "the label path is not the configuration mount point",
        path.ne(interner
            .resolve(
                package
                    .mount_config
                    .map(|s| s.0)
                    .unwrap_or(context.common_interneds().config_mount)
            )
            .map_err(|err| HoneError::UnexpectedError(format!("Interner error: {}", err)))?)
    );

    let path = SmolStr::new(format!("{}/{}", path, consts::BUILD_FILE_NAME));

    let cas_store = context.cas_store();

    // check if the build script exists
    let handle = ctx.request(File { path: path.clone() }.into()).await?;

    let handle = match &**handle.value() {
        ZakoValue::FileResult(file_result) => file_result,
        _ => {
            return Err(HoneError::UnexpectedError(
                "Expected FileResult".to_string(),
            ));
        }
    };

    let data = SmolStr::new(
        String::from_utf8(handle.content.read(cas_store, BlobRange::full()).await?).wrap_err_with(
            || format!("failed to read `{}` as valid utf-8 string", path.as_str()),
        )?,
    );

    let output_hash = handle.content.digest().blake3;

    Ok((
        HashPair {
            input_hash: input_hash.into(),
            output_hash: output_hash.into(),
        },
        ResolveLabelResult {
            target: crate::target::Target::Target(data),
        },
    ))
}

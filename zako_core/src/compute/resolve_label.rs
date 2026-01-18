use ::eyre::{Context, OptionExt};
use ::hone::debug_assert;
use ::smol_str::SmolStr;
use ::zako_digest::blake3::Blake3Hash;
use hone::{HoneResult, error::HoneError, status::HashPair};

use crate::{
    blob_range::BlobRange,
    computer::ZakoComputeContext,
    consts,
    intern::InternedAbsolutePath,
    node::{
        file::File,
        node_value::ZakoValue,
        resolve_label::{ResolveLabel, ResolveLabelResult},
    },
};

pub async fn resolve_label<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    key: &ResolveLabel,
) -> HoneResult<(HashPair, ResolveLabelResult)> {
    todo!()
    /*
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
            .package
            .mount_config
            .map(|s| s.0)
            .unwrap_or(context.common_interneds().config_mount)
    {
        let resolved = package
            .package
            .config
            .resolve(interner)
            .wrap_err("failed to resolve configuration for mount_config target")?;
        return Ok((
            HashPair {
                input_hash: input_hash.into(),
                output_hash: resolved.get_blake3().into(),
            },
            ResolveLabelResult {
                target: crate::target::Target::Configuration(package.package.config.clone()),
            },
        ));
    }

    let path = interner
        .resolve(&key.label.path.0)
        .map_err(|err| HoneError::UnexpectedError(format!("Interner error: {}", err)))?;

    debug_assert!(
        "the label path is not the configuration mount point",
        path.ne(interner
            .resolve(
                package
                    .package
                    .mount_config
                    .map(|s| s.0)
                    .unwrap_or(context.common_interneds().config_mount)
            )
            .map_err(|err| HoneError::UnexpectedError(format!("Interner error: {}", err)))?)
    );

    let path = format!("{}/{}", path, consts::BUILD_FILE_NAME);
    let path_id = InternedAbsolutePath::new(&path, interner)
        .wrap_err("failed to resolve path")?
        .ok_or_eyre("failed to build absolute path")?;

    let cas_store = context.cas_store();

    // check if the build script exists
    let handle = ctx.request(File { path: path_id }.into()).await?;

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
     */
}

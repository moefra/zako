use std::path::Path;

use ::rkyv::Archive;
use camino::Utf8PathBuf;
use eyre::{Context, OptionExt};
use hone::{HoneResult, error::HoneError, status::HashPair};
use zako_digest::blake3::Blake3Hash;

use crate::{
    blob_handle::BlobHandle,
    compute::file,
    computer::ZakoComputeContext,
    config::Configuration,
    configured_project::ConfiguredPackage,
    consts,
    context::BuildContext,
    node::{
        node_key::ZakoKey,
        node_value::ZakoValue,
        parse_manifest::ParseManifest,
        resolve_manifest_script::ResolveManifestScript,
        resolve_package::{ResolvePackage, ResolvePackageResult},
    },
    package::ResolvingPackage,
};

/// Compute and resolve a project file
pub async fn resolve_package<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    key: &ResolvePackage,
) -> HoneResult<(HashPair, ResolvePackageResult)> {
    let raw_ctx = ctx;
    let ctx = ctx.context();
    let interner = ctx.interner();

    let package_id = key
        .package
        .resolved(interner)
        .map_err(|err| HoneError::UnexpectedError(format!("Interner error: {}", err)))?;

    let input_hash: blake3::Hash = (package_id.as_str(), key.source.clone()).get_blake3();

    let interned_path = if let Some(root) = key.root {
        root.into()
    } else {
        *(ctx
            .global_state()
            .package_id_to_path()
            .get(&key.package)
            .ok_or_eyre( format!(
                "the package `{}` is not found in the global state. Only registered package is supported now ",
                package_id.as_str()
            ))?)
    };

    let path_str = interner
        .resolve(interned_path)
        .map_err(|err| HoneError::UnexpectedError(format!("Interner error: {}", err)))?;
    let path = Utf8PathBuf::from(path_str);
    let manifest = path.join(consts::PACKAGE_MANIFEST_FILE_NAME);

    // TODO: Do not read the file into memory, just read the file into a blob handle
    // Issue URL: https://github.com/moefra/zako/issues/28
    let (_, result) = file::read_text(raw_ctx, manifest).await?;

    // build new context

    let new_ctx = BuildContext::new(&path, key.source.clone(), None, ctx.global_state().clone())
        .wrap_err("failed to build new context")?;

    let manifest = raw_ctx
        .request_with_context(
            ZakoKey::ParseManifest(ParseManifest {
                blob_handle: BlobHandle::new_referenced(*result.content.digest()),
            }),
            &new_ctx,
        )
        .await
        .wrap_err("failed to request parse manifest")?;

    let parsed = match manifest.value().as_ref() {
        ZakoValue::ParseManifest(resolved) => resolved,
        _ => {
            return Err(eyre::eyre!("expected parse manifest result").into());
        }
    }
    .clone();

    // TODO: Resolve the configuration and dependencies

    // before intern it, calculate hash
    // only hash result `ResolvedPackage`

    _ = parsed
        .project
        .validate()
        .wrap_err("failed to validate project manifest")?;

    let configuration = Configuration {
        config: parsed
            .project
            .config
            .clone()
            .unwrap_or_default()
            .into_iter()
            .collect(),
    };

    let resolving = ResolvingPackage::new(parsed.project, configuration.resolve(interner)?);

    // TODO: call v8 js script to poll more information
    // Issue URL: https://github.com/moefra/zako/issues/29
    // e.g. engine.execute_manifest_initialize_script(resolving)

    let result = raw_ctx
        .request_with_context(
            ZakoKey::ResolveManifestScript(ResolveManifestScript {
                configure_script: resolving.original.configure_script.clone(),
                package: resolving,
            }),
            &new_ctx,
        )
        .await
        .wrap_err("failed to request resolve manifest script")?;

    let resolving = match &*result.into_value() {
        ZakoValue::ResolveManifestScript(resolved) => resolved.package.clone(),
        _ => {
            return Err(eyre::eyre!("expected resolve manifest script result").into());
        }
    };

    let resolved = resolving.resolve(interner).map_err(|err| {
        eyre::eyre!(err).wrap_err(format!(
            "error while resolving project {:?}, path {:?}",
            &key.package, &path
        ))
    })?;

    let output_hash = resolved.get_blake3_hash(interner)?;

    let configured = ConfiguredPackage {
        source: new_ctx.package_source().clone(),
        package: resolved,
    };

    return Ok((
        HashPair {
            input_hash: input_hash.into(),
            output_hash: output_hash,
        },
        ResolvePackageResult {
            package: configured,
        },
    ));
}

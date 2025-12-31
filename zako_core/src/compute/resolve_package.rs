use std::path::Path;

use ::rkyv::Archive;
use camino::Utf8PathBuf;
use eyre::{Context, OptionExt};
use hone::{HoneResult, error::HoneError, status::HashPair};
use zako_digest::blake3_hash::Blake3Hash;

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

    let raw_source_blake3 = key.source.clone().get_blake3();
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

    // TODO: DO not read the file into memory, just read the file into a blob handle
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
            Err(eyre::eyre!("expected parse manifest result"))?;
            unreachable!()
        }
    }
    .clone();
    let raw_package_blake3 = parsed.project.get_blake3();

    // TODO: Resolve the configuration and dependencies

    // before intern it, calculate hash
    // only hash result `ResolvedPackage`

    _ = parsed.project.pre_resolve()?;

    let resolving = ResolvingPackage::new(
        parsed.project,
        Configuration::from(parsed.project.config.unwrap_or_default()).resolve(interner)?,
    );

    // TODO: call v8 js script to poll more information
    // e.g. engine.execute_manifest_initialize_script(resolving)

    let resolved = resolving.resolve(&new_ctx).map_err(|err| {
        eyre::eyre!(err).wrap_err(format!(
            "while resolving project {:?}, path {:?}",
            &key.package, &path
        ))
    })?;

    let configured = ConfiguredPackage {
        raw_package_blake3: resolved.get_blake3(),
        raw_source_blake3: raw_source_blake3,
        source_root_blake3: interned_path.get_blake3(),
        source: new_ctx.package_source().clone(),
        package: resolved,
        source_root: interned_path,
    };

    let output = resolved.get_blake3();

    return Ok((
        HashPair {
            input_hash: input_hash.into(),
            output_hash: output_hash.into(),
        },
        ResolvePackageResult {
            package: configured,
        },
    ));
}

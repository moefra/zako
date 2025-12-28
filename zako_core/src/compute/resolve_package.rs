use std::path::Path;

use camino::Utf8PathBuf;
use eyre::{Context, OptionExt};
use hone::{HoneResult, error::HoneError, status::HashPair};
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    blob_handle::BlobHandle,
    computer::ZakoComputeContext,
    configured_project::ConfiguredPackage,
    consts,
    context::BuildContext,
    node::{
        node_key::ZakoKey,
        node_value::ZakoValue,
        parse_manifest::ParseManifest,
        resolve_package::{ResolvePackage, ResolvePackageResult},
    },
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

    let path_id = if let Some(root) = key.root {
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
        .resolve(path_id.interned)
        .map_err(|err| HoneError::UnexpectedError(format!("Interner error: {}", err)))?;
    let path = Path::new(path_str);
    let path =
        Utf8PathBuf::try_from(path.canonicalize().map_err(|e| eyre::eyre!(e))?).map_err(|e| {
            HoneError::IOError(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidFilename,
                    format!(
                        "Invalid filename: {:?}(contains non-utf8 characters: {:?})",
                        &path_str, e
                    ),
                ),
                path_str.to_string(),
            )
        })?;
    let manifest = path.join(consts::PACKAGE_MANIFEST_FILE_NAME);
    let manifest_digest = ctx
        .cas_store()
        .get_local_cas()
        .input_file(&manifest)
        .await
        .wrap_err("failed to input file to local cas")?;

    // build new context
    let new_ctx = BuildContext::new(&path, key.source.clone(), None, ctx.global_state().clone())
        .wrap_err("failed to build new context")?;

    let manifest = raw_ctx
        .request_with_context(
            ZakoKey::ParseManifest(ParseManifest {
                blob_handle: BlobHandle::new_referenced(manifest_digest),
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

    // TODO: Resolve the configuration and dependencies

    // before intern it, calculate hash
    // only hash result `ResolvedPackage`
    let output_hash = parsed.project.get_blake3();

    let resolved = parsed.project.resolve(&new_ctx, &path).map_err(|err| {
        eyre::eyre!(err).wrap_err(format!(
            "while resolving project {:?}, path {:?}",
            &key.package, &path
        ))
    })?;

    return Ok((
        HashPair {
            input_hash: input_hash.into(),
            output_hash: output_hash.into(),
        },
        ResolvePackageResult {
            package: ConfiguredPackage {
                source: new_ctx.package_source().clone(),
                package: resolved,
                source_root: path_id,
                raw_source: key.source.clone(),
            },
        },
    ));
}

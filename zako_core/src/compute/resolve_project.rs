use std::path::{Path, PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use eyre::{Context, OptionExt};
use hone::{
    HoneResult,
    error::HoneError,
    status::{HashPair, NodeData},
};
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    blob_handle::BlobHandle,
    computer::ZakoComputeContext,
    consts,
    context::BuildContext,
    node::{
        node_key::ZakoKey,
        node_value::ZakoValue,
        parse_manifest::ParseManifest,
        resolve_project::{ResolveProject, ResolveProjectResult},
    },
    project::{Project, ResolvedProject},
};

/// Compute and resolve a project file
pub async fn compute_resolve_project<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    key: &ResolveProject,
) -> HoneResult<(HashPair, ResolveProjectResult)> {
    let raw_ctx = ctx;
    let ctx = ctx.context();
    let interner = ctx.interner();

    let package_id_to_path = key.package.resolved(interner);

    let input_hash: blake3::Hash = package_id_to_path.get_blake3();

    let path = if let Some(root) = key.root {
        root.into()
    } else {
        *(ctx
            .global_state()
            .package_id_to_path()
            .get(&key.package)
            .ok_or_eyre(format!(
                "the package `{}` is not found in the global state. Only registered package is supported now ",
                package_id_to_path.as_str()
            ))?)
    };

    let path = interner.resolve(path.interned);
    let path = Path::new(path);
    let mut path = path.canonicalize().map_err(|e| eyre::eyre!(e))?;
    let path = Utf8PathBuf::try_from(path).map_err(|e| {
        HoneError::IOError(
            std::io::Error::new(
                std::io::ErrorKind::InvalidFilename,
                format!("Invalid filename: {:?}({:?})", path, e),
            ),
            path.to_string_lossy().to_string(),
        )
    })?;
    let manifest = path.join(consts::PROJECT_MANIFEST_FILE_NAME);
    let manifest_digest = ctx
        .cas_store()
        .get_local_cas()
        .input_file(&manifest)
        .await
        .wrap_err("failed to input file to local cas")?;

    // build new context
    let new_ctx = BuildContext::new(&path, key.source, None, ctx.global_state().clone())
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

    todo!("Resolve the project, resolve configuration");

    // TODO: Resolve the project, resolve configuration

    /*
    Ok((
        HashPair::new(input_hash.into(), output_hash.into()),
        ResolveProjectResult {
            root: path.to_string_lossy().into(),
            project: resolved_project,
        },
    ))
    */
}

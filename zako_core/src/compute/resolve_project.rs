use std::path::{Path, PathBuf};

use eyre::OptionExt;
use hone::{
    HoneResult,
    status::{HashPair, NodeData},
};
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    computer::ZakoComputeContext,
    consts,
    context::BuildContext,
    node::{
        node_key::ZakoKey,
        node_value::ZakoValue,
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

    let path = *(ctx
        .global_state()
        .package_id_to_path()
        .get(&key.package)
        .ok_or_eyre(format!(
            "the package `{}` is not found in the global state. Only registered package is supported now ",
            package_id_to_path.as_str()
        ))?);

    let path = interner.resolve(path.interned);
    let path = Path::new(path);
    let mut path = path.canonicalize().map_err(|e| eyre::eyre!(e))?;
    path.push(consts::PROJECT_MANIFEST_FILE_NAME);

    // TODO: Use raw_ctx.request() to read the project manifest file
    let content = std::fs::read_to_string(&path).map_err(|e| eyre::eyre!(e))?;
    let project: Project = toml::from_str(content.as_ref()).map_err(|e| eyre::eyre!(e))?;

    let output_hash = project.get_blake3();

    let resolved_project = project.resolve(ctx, &path).map_err(|e| eyre::eyre!(e))?;

    Ok((
        HashPair::new(input_hash.into(), output_hash.into()),
        ResolveProjectResult {
            root: path.to_string_lossy().into(),
            project: resolved_project,
        },
    ))
}

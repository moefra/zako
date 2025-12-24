use std::{path::Path, sync::Arc};

use hone::{
    HoneResult,
    error::HoneError,
    status::{HashPair, NodeData},
};
#[cfg(unix)]
use tokio::fs;
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    computer::ZakoComputeContext,
    context::BuildContext,
    node::{
        file::{File, FileResult},
        node_key::ZakoKey,
        node_value::ZakoValue,
    },
    path::interned::InternedNeutralPath,
};

pub async fn compute_file<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    key: &File,
) -> HoneResult<(HashPair, FileResult)> {
    let path = &key.path;

    let build_ctx = ctx.context();
    let interner = build_ctx.interner();
    let abs_root = interner.resolve(build_ctx.project_root().interned);
    let path_str = interner.resolve(path.interned());
    let physical_path = Path::new(abs_root).join(path_str);

    let bytes = tokio::fs::read(&physical_path).await.map_err(|e| {
        HoneError::Other(eyre::Report::msg(format!(
            "Failed to read source file {:?}: {}",
            physical_path, e
        )))
    })?;

    #[cfg(unix)]
    let is_exec = std::os::unix::fs::MetadataExt::mode(
        &fs::metadata(&physical_path)
            .await
            .map_err(|err| HoneError::IOError(err, physical_path.to_string_lossy().to_string()))?,
    ) & 0o111
        != 0;
    #[cfg(not(unix))]
    let is_exec = true;

    let handle = build_ctx
        .cas_store()
        .put_bytes(bytes)
        .await
        .map_err(|err| {
            HoneError::Other(eyre::Report::msg(format!(
                "Failed to put bytes into cas store: {}",
                err
            )))
        })?;

    // The hash of input path is input hash
    let input_hash = blake3::hash(path_str.as_bytes());

    Ok((
        HashPair::new(handle.digest().blake3.into(), input_hash.into()),
        FileResult {
            path: path.clone(),
            is_executable: is_exec,
            content: handle.clone(),
        },
    ))
}

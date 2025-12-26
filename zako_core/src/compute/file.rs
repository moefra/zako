use std::{path::Path, sync::Arc};

use camino::{Utf8Path, Utf8PathBuf};
use hone::{
    HoneResult,
    error::HoneError,
    status::{HashPair, NodeData},
};
use is_executable::is_executable;
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    blob_handle::BlobHandle,
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
    let physical_path = Utf8Path::new(abs_root).join(path_str);

    if std::fs::exists(physical_path.as_path())
        .map_err(|e| HoneError::IOError(e, physical_path.to_string()))?
    {
        return Err(HoneError::IOError(
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
            physical_path.to_string(),
        ));
    }

    let is_symlink = {
        match std::fs::symlink_metadata(physical_path.as_path()) {
            Ok(meta) => meta.is_symlink(),
            Err(e) => return Err(HoneError::IOError(e, physical_path.to_string())),
        }
    };

    let is_exec = is_executable(&physical_path);

    let local_cas = build_ctx.cas_store().get_local_cas();

    let digest = local_cas
        .input_file(&physical_path)
        .await
        .map_err(|e| HoneError::IOError(e, physical_path.to_string()))?;

    // The hash of input path is input hash
    let input_hash = blake3::hash(path_str.as_bytes());

    Ok((
        HashPair {
            output_hash: digest.blake3.into(),
            input_hash: input_hash.into(),
        },
        FileResult {
            is_executable: is_exec,
            content: BlobHandle::new_referenced(digest.clone()),
            is_symlink,
        },
    ))
}

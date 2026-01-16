use ::std::path::Path;

use ::eyre::{Context, ContextCompat};
use ::hone::{node::NodeValue, status::NodeData};
use camino::Utf8Path;
use hone::{HoneResult, error::HoneError, status::HashPair};
use is_executable::is_executable;

use crate::{
    blob_handle::BlobHandle,
    blob_range::BlobRange,
    computer::ZakoComputeContext,
    intern::InternedAbsolutePath,
    node::{
        file::{File, FileResult},
        node_value::ZakoValue,
    },
};

pub async fn file<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    key: &File,
) -> HoneResult<(HashPair, FileResult)> {
    let path = &key.path;

    let build_ctx = ctx.context();
    let interner = build_ctx.interner();
    let abs_root = interner
        .resolve(build_ctx.project_root())
        .map_err(|err| HoneError::UnexpectedError(format!("Interner error: {}", err)))?;
    let path_str = interner.resolve(path).wrap_err("failed to resolve path")?;
    let physical_path = Utf8Path::new(abs_root).join(path_str);

    if std::fs::exists(physical_path.as_path())
        .map_err(|e| HoneError::IOError(e, physical_path.to_string()))?
    {
        return Err(HoneError::IOError(
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
            physical_path.to_string(),
        ));
    }

    let is_symlink = std::fs::symlink_metadata(physical_path.as_path())
        .map(|meta| meta.is_symlink())
        .unwrap_or(false);

    let is_executable = (!is_symlink) && is_executable(&physical_path);

    let local_cas = build_ctx.cas_store().get_local_cas();

    let digest = local_cas
        .input_file(&physical_path, is_symlink)
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
            is_executable,
            content: BlobHandle::new_referenced(digest.clone()),
            is_symlink,
        },
    ))
}

pub async fn read_file<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    path: impl AsRef<Path>,
) -> HoneResult<(Vec<u8>, FileResult)> {
    let path = path.as_ref();

    let path = Utf8Path::from_path(path).ok_or_else(|| {
        return HoneError::IOError(
            std::io::Error::new(
                ::std::io::ErrorKind::InvalidFilename,
                eyre::eyre!("the file path contains invalid utf-8 character"),
            ),
            format!("{:?}", &path),
        );
    })?;

    read_file_utf8(ctx, path).await
}

pub async fn read_file_utf8<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    path: impl AsRef<Utf8Path>,
) -> HoneResult<(Vec<u8>, FileResult)> {
    let interner = ctx.context().interner();
    let path = path.as_ref();
    let path = InternedAbsolutePath::new(path, interner)
        .wrap_err_with(|| eyre::eyre!("failed to intern the path {:?}", path))?;
    let handle = ctx.request(File { path }.into()).await?;
    let handle = match &**handle {
        ZakoValue::FileResult(result) => result,
        _ => {
            return Err(HoneError::UnexpectedError(format!(
                "unexpected node data: {:?}",
                handle
            )));
        }
    };
    let read = handle
        .content
        .read(ctx.context().cas_store(), BlobRange::full())
        .await?;
    Ok((read, handle.clone()))
}

pub async fn read_text<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    path: impl AsRef<Path>,
) -> HoneResult<(String, FileResult)> {
    let (read, handle) = read_file(ctx, path).await?;
    Ok((
        String::from_utf8(read).map_err(|e| {
            HoneError::UnexpectedError(format!(
                "failed to read file as valid utf-8 string: {:?}",
                e
            ))
        })?,
        handle,
    ))
}

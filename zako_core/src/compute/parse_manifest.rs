use hone::{HoneResult, status::HashPair};
use zako_digest::blake3::Blake3Hash;

use crate::{
    blob_range::BlobRange,
    computer::ZakoComputeContext,
    node::parse_manifest::{ParseManifest, ParseManifestResult},
    package::Package,
};

pub async fn prase_manifest<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    key: &ParseManifest,
) -> HoneResult<(HashPair, ParseManifestResult)> {
    let blob_handle = key.blob_handle.clone();

    let read = blob_handle
        .read(ctx.context().cas_store(), BlobRange::full())
        .await?;

    let project: Package = toml::from_slice(&read).map_err(|e| eyre::eyre!(e))?;

    Ok((
        HashPair {
            output_hash: project.get_blake3().into(),
            input_hash: blob_handle.digest().blake3,
        },
        ParseManifestResult { project },
    ))
}

use std::{path::Path, sync::Arc};

use eyre::Context;
use hone::{
    HoneResult,
    error::HoneError,
    status::{HashPair, NodeData},
};
use oxc_span::SourceType;
#[cfg(unix)]
use tokio::fs;
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    blob_range::BlobRange,
    computer::ZakoComputeContext,
    context::BuildContext,
    node::{
        node_key::ZakoKey,
        node_value::ZakoValue,
        transpile_ts::{TranspileTs, TranspileTsResult},
    },
    path::interned::InternedNeutralPath,
    worker::oxc_worker::OxcTranspilerInput,
};

pub async fn compute_transpile_ts<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    key: &TranspileTs,
) -> HoneResult<(HashPair, TranspileTsResult)> {
    let code = key.code.clone();
    let code = ctx
        .context()
        .cas_store()
        .read(
            code.digest(),
            &BlobRange::new(0, None).map_err(|err| HoneError::Other(err.into()))?,
        )
        .await
        .wrap_err_with(|| {
            format!(
                "failed to read typescript code {:?} from cas store",
                key.code.digest(),
            )
        })?;

    let input_hash = code.get_blake3();

    let result = ctx
        .context()
        .oxc_workers_pool()
        .submit(
            OxcTranspilerInput {
                source_text: String::from_utf8(code)
                    .wrap_err_with(|| {
                        format!(
                            "failed to convert typescript code {:?} to utf-8 string",
                            key.code.digest(),
                        )
                    })?
                    .to_string(),
                source_name: key.name.clone(),
                source_type: SourceType::ts(),
            },
            ctx.cancel_token(),
        )
        .await
        .wrap_err_with(|| {
            format!(
                "failed to submit typescript code `{:?}` to oxc workers",
                key.code.digest(),
            )
        })?
        .wrap_err_with(|| {
            format!(
                "failed to transpile typescript code {:?}",
                key.code.digest()
            )
        })?;

    let output = TranspileTsResult {
        code: result.code,
        source_map: result.map,
    };

    let output_hash = output.get_blake3();

    Ok((
        HashPair {
            output_hash: output_hash.into(),
            input_hash: input_hash.into(),
        },
        output,
    ))
}

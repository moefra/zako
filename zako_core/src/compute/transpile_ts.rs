use eyre::Context;
use hone::{HoneResult, error::HoneError, status::HashPair};
use oxc_span::SourceType;
use zako_digest::blake3::Blake3Hash;

use crate::{
    blob_handle::BlobHandle,
    blob_range::BlobRange,
    computer::ZakoComputeContext,
    node::{
        node_value::ZakoValue,
        transpile_ts::{TranspileTs, TranspileTsResult},
    },
    worker::oxc_worker::OxcTranspilerInput,
};

pub async fn transpile_ts<'c>(
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

pub async fn transpile_ts_string<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    name: String,
    blob_handle: BlobHandle,
) -> HoneResult<TranspileTsResult> {
    let result = ctx
        .request(
            TranspileTs {
                name: name,
                code: blob_handle,
            }
            .into(),
        )
        .await?;

    let result = match &**result.value() {
        ZakoValue::TranspileTs(result) => result,
        _ => {
            return Err(HoneError::UnexpectedError(
                "Unexpected node value".to_string(),
            ));
        }
    };

    Ok(result.clone())
}

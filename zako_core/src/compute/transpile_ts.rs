use std::{path::Path, sync::Arc};

use eyre::Context;
use hone::{HoneResult, error::HoneError, status::NodeData};
use oxc_span::SourceType;
#[cfg(unix)]
use tokio::fs;
use xxhash_rust::xxh3::xxh3_128;
use zako_digest::hash::XXHash3;

use crate::{
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
) -> HoneResult<(u128, u128, TranspileTsResult)> {
    let code = key.code;
    let code = ctx
        .context()
        .cas_store()
        .read(&code.into(), 0, None)
        .await
        .wrap_err_with(|| {
            format!(
                "failed to read typescript code {:?} from cas store",
                key.code.hash,
            )
        })?;

    let input_hash = code.xxhash3_128();

    let result = ctx
        .context()
        .oxc_workers_pool()
        .submit(
            OxcTranspilerInput {
                source_text: String::from_utf8(code)
                    .wrap_err_with(|| {
                        format!(
                            "failed to convert typescript code {:?} to utf-8 string",
                            key.code.hash,
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
                key.code.hash
            )
        })?
        .wrap_err_with(|| format!("failed to transpile typescript code {:?}", key.code.hash))?;

    let output = TranspileTsResult {
        code: result.code,
        source_map: result.map,
    };

    let output_hash = output.xxhash3_128();

    Ok((input_hash, output_hash, output))
}

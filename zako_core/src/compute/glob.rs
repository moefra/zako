use std::{path::Path, sync::Arc};

use eyre::OptionExt;
use hone::{HoneResult, status::NodeData};
use zako_digest::hash::XXHash3;

use crate::{
    computer::ZakoComputeContext, context::BuildContext, intern::InternedAbsolutePath,
    node_value::ZakoValue, path::NeutralPath, pattern::InternedPattern, resource::ResourceRequest,
};

/// Compute glob results for a given base path and pattern
pub async fn compute_glob<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    base_path: &InternedAbsolutePath,
    pattern: &InternedPattern,
) -> HoneResult<NodeData<BuildContext, ZakoValue>> {
    let old_data = ctx.old_data();
    let ctx = ctx.context();
    let _resource = ctx.resource_pool().occupy(ResourceRequest::cpu(1));
    let base_path = ctx.interner().resolve(base_path.interned());
    let base_path = Path::new(base_path);
    let interner = ctx.interner();

    let input_hash = {
        let mut input_hasher = xxhash_rust::xxh3::Xxh3::new();
        base_path.hash_into(&mut input_hasher);
        pattern.hash_into(&mut input_hasher);
        input_hasher.digest128()
    };

    let mut result = pattern.walk(interner, &base_path, 1).map_err(|err| {
        eyre::Report::new(err).wrap_err(format!(
            "failed to walk directory `{:?}` with pattern `{:?}`",
            base_path,
            pattern.resolve(&interner) // To provide debug information
        ))
    })?;
    // IMPORTANT: sort the result to ensure the same order
    result.sort();

    let mut interned_neutral_result = Vec::with_capacity(result.len());
    let mut hasher = xxhash_rust::xxh3::Xxh3::new();

    for path in result {
        // Convert path to base_path relative neutral path
        let diff = pathdiff::diff_paths(&path, &base_path).ok_or_else(|| {
            eyre::Report::msg(format!(
                "Failed to compute relative path `{:?}`,base path:{:?}",
                path, base_path,
            ))
        })?;
        let neutral_path = NeutralPath::new(diff.to_string_lossy().to_string()).map_err(|err| {
            eyre::Report::new(err).wrap_err(format!(
                "get an path error when construct NeutralPath, path:`{:?}`,base path:{:?}, diff:{:?}",
                path, base_path, diff,
            ))
        })?;
        neutral_path.hash_into(&mut hasher);
        let neutral_path = neutral_path.intern(ctx.interner());
        interned_neutral_result.push(neutral_path);
    }

    let output_hash = hasher.digest128();

    if let Some(old) = old_data {
        if old.output_xxhash3() == output_hash {
            // 结果没变，复用旧数据！
            return Ok(NodeData::new(old.value().clone(), output_hash, input_hash));
        }
    }

    Ok(NodeData::new(
        Arc::new(ZakoValue::Glob(interned_neutral_result)),
        output_hash,
        input_hash,
    ))
}

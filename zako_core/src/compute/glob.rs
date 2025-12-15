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

    let result = pattern
        .walk(ctx.interner(), &base_path, 1)
        .map_err(|err| eyre::Report::new(err))?;

    let mut neutral_result = Vec::with_capacity(result.len());

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
                "get an path error when glob, path:`{:?}`,base path:{:?}, diff:{:?}",
                path, base_path, diff,
            ))
        })?;
        neutral_path.hash_into(&mut hasher);
        let neutral_path = neutral_path.intern(ctx.interner());
        neutral_result.push(neutral_path);
    }

    Ok(NodeData::new(
        Arc::new(ZakoValue::Glob(neutral_result)),
        hasher.digest128(), // output hash
        hasher.digest128(), // input hash
    ))
}

use std::{path::Path, sync::Arc};

use blake3::hazmat::HasherExt;
use camino::Utf8Path;
use eyre::OptionExt;
use eyre::eyre;
use hone::{
    HoneResult,
    status::{HashPair, NodeData},
};
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    computer::ZakoComputeContext,
    context::BuildContext,
    intern::InternedAbsolutePath,
    node::{
        glob::{Glob, GlobResult},
        node_value::ZakoValue,
    },
    path::NeutralPath,
    pattern::InternedPattern,
    resource::ResourceRequest,
};

/// Compute glob results for a given base path and pattern
pub async fn compute_glob<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    glob: &Glob,
) -> HoneResult<(HashPair, GlobResult)> {
    let base_path = &glob.base_path;
    let pattern = &glob.pattern;

    let old_data = ctx.old_data();
    let ctx = ctx.context();
    let _resource = ctx.resource_pool().occupy(ResourceRequest::cpu(1));
    let base_path = ctx.interner().resolve(base_path.interned);
    let base_path = Utf8Path::new(base_path);
    let interner = ctx.interner();

    let resolved_pattern = pattern.resolve(interner);

    let input_hash = {
        let mut input_hasher = blake3::Hasher::new();
        base_path.hash_into_blake3(&mut input_hasher);
        resolved_pattern.hash_into_blake3(&mut input_hasher);
        input_hasher.finalize()
    };

    let mut result = pattern
        .walk(interner, &base_path.as_std_path(), 1)
        .map_err(|err| {
            eyre::Report::new(err).wrap_err(format!(
                "failed to walk directory `{:?}` with pattern `{:?}`",
                base_path,
                pattern.resolve(&interner) // To provide debug information
            ))
        })?;
    // IMPORTANT: sort the result to ensure the same order
    result.sort();

    let mut interned_neutral_result = Vec::with_capacity(result.len());
    let mut hasher = blake3::Hasher::new();

    for path in result {
        // Convert path to base_path relative neutral path
        let diff = pathdiff::diff_paths(&path, &base_path).ok_or_else(|| {
            eyre::Report::msg(format!(
                "Failed to compute relative path `{:?}`,base path:{:?}",
                path, base_path,
            ))
        })?;
        let diff = Utf8Path::from_path(&diff).ok_or_else(|| {
            eyre!(
                "glob: failed to convert path {:?} to Utf8Path, base path:{:?}",
                diff,
                base_path
            )
        })?;
        let neutral_path = NeutralPath::from_path(&diff).map_err(|err| {
            eyre::Report::new(err).wrap_err(format!(
                "get an path error when construct NeutralPath, path:{:?} ,base path:{:?}, diff:{:?}",
                path, base_path, diff,
            ))
        })?;
        neutral_path.hash_into_blake3(&mut hasher);
        let neutral_path = neutral_path.intern(ctx.interner());
        interned_neutral_result.push(neutral_path);
    }

    let output_hash = hasher.finalize();

    /*
    TODO: Reuse old data
    Issue URL: https://github.com/moefra/zako/issues/17
    if let Some(old) = old_data {
        if old.output_xxhash3() == output_hash {
            // 结果没变，复用旧数据！

        }
    }
    */

    return Ok((
        HashPair::new(output_hash.into(), input_hash.into()),
        GlobResult {
            paths: interned_neutral_result,
        },
    ));
}

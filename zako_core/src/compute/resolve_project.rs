use std::path::PathBuf;

use hone::{HoneResult, status::NodeData};

use crate::{
    computer::ZakoComputeContext,
    context::BuildContext,
    node::{node_value::ZakoValue, resolve_project::ResolveProject},
    project::ResolvedProject,
};

/// Compute and resolve a project file
pub async fn compute_resolve_project<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    key: &ResolveProject,
) -> HoneResult<(HashPair, ResolveProject)> {
    // TODO: Implement project resolution logic here
    // Issue URL: https://github.com/moefra/zako/issues/11
    // 1. Read project file (zako.toml)
    // 2. Parse configuration
    // 3. Create BuildContext from config
    // 4. Return as ZakoValue::ResolvedProject

    todo!("Implement project resolution")
}

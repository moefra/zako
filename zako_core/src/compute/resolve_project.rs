use std::path::PathBuf;

use hone::{HoneResult, status::NodeData};

use crate::{
    computer::ZakoComputeContext,
    node_value::ZakoValue,
    context::BuildContext,
};

/// Compute and resolve a project file
pub async fn compute_resolve_project<'c>(
    _ctx: &'c ZakoComputeContext<'c>,
    _path: &PathBuf,
) -> HoneResult<NodeData<BuildContext, ZakoValue>> {
    // TODO: Implement project resolution logic here
    // 1. Read project file (zako.json/zako.jsonc)
    // 2. Parse configuration
    // 3. Create BuildContext from config
    // 4. Return as ZakoValue::ResolvedProject

    todo!("Implement project resolution")
}

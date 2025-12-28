use ::hone::{HoneResult, status::HashPair};

use crate::{
    computer::ZakoComputeContext,
    node::resolve_manifest_script::{ResolveManifestScript, ResolveManifestScriptResult},
};

pub async fn resolve_manifest_script<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    key: &ResolveManifestScript,
) -> HoneResult<(HashPair, ResolveManifestScriptResult)> {
    todo!()
}

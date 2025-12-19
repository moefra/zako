use async_trait::async_trait;
use hone::{HoneResult, context::Context, status::NodeData};

use crate::{
    compute::{compute_file, compute_glob, compute_resolve_project},
    context::BuildContext,
    node_key::ZakoKey,
    node_value::ZakoValue,
    path::interned::InternedNeutralPath,
};

#[derive(Debug)]
pub struct Compuer {}

pub type ZakoComputer = dyn hone::context::Computer<BuildContext, ZakoKey, ZakoValue>;
pub type ZakoComputeContext<'c> = Context<'c, BuildContext, ZakoKey, ZakoValue>;
pub type ZakoResult = HoneResult<NodeData<BuildContext, ZakoValue>>;

#[async_trait]
impl hone::context::Computer<BuildContext, ZakoKey, ZakoValue> for Compuer {
    async fn compute<'c>(
        &self,
        ctx: &'c ZakoComputeContext<'c>,
    ) -> HoneResult<NodeData<BuildContext, ZakoValue>> {
        match ctx.this() {
            ZakoKey::Glob { base_path, pattern } => compute_glob(ctx, &base_path, &pattern).await,
            ZakoKey::ResolveProject { path } => compute_resolve_project(ctx, &path).await,
            ZakoKey::File { path } => compute_file(ctx, &path).await,
        }
    }
}

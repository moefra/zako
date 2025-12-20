use std::sync::Arc;

use async_trait::async_trait;
use hone::{HoneResult, context::Context, status::NodeData};

use crate::{
    compute::{compute_file, compute_glob, compute_resolve_project, compute_transpile_ts},
    context::BuildContext,
    node::{node_key::ZakoKey, node_value::ZakoValue},
    path::interned::InternedNeutralPath,
};

#[derive(Debug)]
pub struct Computer {}

pub type ZakoComputer = dyn hone::context::Computer<BuildContext, ZakoKey, ZakoValue>;
pub type ZakoComputeContext<'c> = Context<'c, BuildContext, ZakoKey, ZakoValue>;
pub type ZakoResult = HoneResult<NodeData<BuildContext, ZakoValue>>;

#[async_trait]
impl hone::context::Computer<BuildContext, ZakoKey, ZakoValue> for Computer {
    async fn compute<'c>(
        &self,
        ctx: &'c ZakoComputeContext<'c>,
    ) -> HoneResult<NodeData<BuildContext, ZakoValue>> {
        match ctx.this() {
            ZakoKey::Glob(key) => compute_glob(ctx, key).await.map(|result| {
                NodeData::new(result.0, result.1, Arc::new(ZakoValue::Glob(result.2)))
            }),
            ZakoKey::ResolveProject(key) => compute_resolve_project(ctx, key).await.map(|result| {
                NodeData::new(
                    result.0,
                    result.1,
                    Arc::new(ZakoValue::ResolveProject(result.2)),
                )
            }),
            ZakoKey::File(key) => compute_file(ctx, key).await.map(|result| {
                NodeData::new(
                    result.0,
                    result.1,
                    Arc::new(ZakoValue::FileResult(result.2)),
                )
            }),
            ZakoKey::TranspileTs(key) => compute_transpile_ts(ctx, key).await.map(|result| {
                NodeData::new(
                    result.0,
                    result.1,
                    Arc::new(ZakoValue::TranspileTsResult(result.2)),
                )
            }),
        }
    }
}

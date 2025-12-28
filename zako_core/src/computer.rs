use std::sync::Arc;

use ::tracing::{Instrument, instrument, trace_span};
use async_trait::async_trait;
use hone::{HoneResult, context::Context, status::NodeData};

use crate::{
    compute::{
        file, glob, prase_manifest, resolve_label, resolve_manifest_script, resolve_package,
        transpile_ts,
    },
    context::BuildContext,
    node::{node_key::ZakoKey, node_value::ZakoValue},
};

#[derive(Debug)]
pub struct Computer {}

pub type ZakoComputer = dyn hone::context::Computer<BuildContext, ZakoKey, ZakoValue>;
pub type ZakoComputeContext<'c> = Context<'c, BuildContext, ZakoKey, ZakoValue>;
pub type ZakoResult = HoneResult<NodeData<BuildContext, ZakoValue>>;

#[async_trait]
impl hone::context::Computer<BuildContext, ZakoKey, ZakoValue> for Computer {
    #[instrument(name = "compute()", level = "trace")]
    async fn compute<'c>(
        &self,
        ctx: &'c ZakoComputeContext<'c>,
    ) -> HoneResult<NodeData<BuildContext, ZakoValue>> {
        let span = trace_span!("dispatched compute()", key = ?ctx.this());

        match ctx.this() {
            ZakoKey::Glob(key) => glob(ctx, key)
                .instrument(span)
                .await
                .map(|result| NodeData::new(result.0, Arc::new(ZakoValue::Glob(result.1)))),
            ZakoKey::ResolvePackage(key) => {
                resolve_package(ctx, key)
                    .instrument(span)
                    .await
                    .map(|result| {
                        NodeData::new(result.0, Arc::new(ZakoValue::ResolvePackage(result.1)))
                    })
            }
            ZakoKey::File(key) => file(ctx, key)
                .instrument(span)
                .await
                .map(|result| NodeData::new(result.0, Arc::new(ZakoValue::FileResult(result.1)))),
            ZakoKey::TranspileTs(key) => transpile_ts(ctx, key)
                .instrument(span)
                .await
                .map(|result| NodeData::new(result.0, Arc::new(ZakoValue::TranspileTs(result.1)))),
            ZakoKey::ParseManifest(key) => {
                prase_manifest(ctx, key)
                    .instrument(span)
                    .await
                    .map(|result| {
                        NodeData::new(result.0, Arc::new(ZakoValue::ParseManifest(result.1)))
                    })
            }
            ZakoKey::ResolveLabel(key) => resolve_label(ctx, key)
                .instrument(span)
                .await
                .map(|result| NodeData::new(result.0, Arc::new(ZakoValue::ResolveLabel(result.1)))),
            ZakoKey::ResolveManifestScript(key) => resolve_manifest_script(ctx, key)
                .instrument(span)
                .await
                .map(|result| {
                    NodeData::new(
                        result.0,
                        Arc::new(ZakoValue::ResolveManifestScript(result.1)),
                    )
                }),
        }
    }
}

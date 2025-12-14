use std::{f32::consts::E, pin::Pin};

use async_trait::async_trait;
use hone::{HoneResult, context::Context, error::HoneError, status::NodeData};

use crate::{context::BuildContext, node_key::ZakoKey, node_value::ZakoValue};

#[derive(Debug)]
pub struct Compuer {}

pub type ZakoComputer = dyn hone::context::Computer<BuildContext, ZakoKey, ZakoValue>;
pub type ZakoComputeContext<'c> = Context<'c, BuildContext, ZakoKey, ZakoValue>;

#[async_trait]
impl hone::context::Computer<BuildContext, ZakoKey, ZakoValue> for Compuer {
    async fn compute<'c>(
        &self,
        ctx: &'c ZakoComputeContext<'c>,
    ) -> HoneResult<NodeData<BuildContext, ZakoValue>> {
        todo!()
    }
}

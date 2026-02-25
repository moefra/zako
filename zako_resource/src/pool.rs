use std::sync::Arc;

use parking_lot::Mutex;
use zako_shared::FastMap;

use crate::{
    ResourceDescriptor, ResourcePoolError,
    allocation::{ResourceGrant, ResourceRequest},
    resource_key::ResourceKey,
    shares::ResourceUnitShares,
};

#[derive(Debug)]
pub struct ResourceHolder {
    span: tracing::Span,
    grant: ResourceGrant,
    pool: Arc<dyn crate::ResourcePool>,
}

impl ResourceHolder {
    pub fn new(
        span: tracing::Span,
        grant: ResourceGrant,
        pool: Arc<dyn crate::ResourcePool>,
    ) -> Box<dyn crate::ResourceHolder> {
        Box::new(Self { span, grant, pool })
    }
}

impl crate::ResourceHolder for ResourceHolder {
    fn grant(&self) -> &ResourceGrant {
        &self.grant
    }
    fn span(&self) -> &tracing::Span {
        &self.span
    }
}

impl Drop for ResourceHolder {
    fn drop(&mut self) {
        self.pool.deallocate(self);
    }
}

#[derive(Debug)]
struct ResourcePoolInner {
    resources: FastMap<ResourceKey, ResourceDescriptor>,
    rest: FastMap<ResourceKey, ResourceUnitShares>,
}

#[derive(Debug)]
pub struct ResourcePool {
    inner: Mutex<ResourcePoolInner>,
}

#[async_trait::async_trait]
impl crate::ResourcePool for ResourcePool {
    async fn allocate(
        &self,
        request: &ResourceRequest,
    ) -> Result<Box<dyn crate::ResourceHolder>, ResourcePoolError> {
        todo!()
    }
    fn try_allocate(
        &self,
        request: &ResourceRequest,
    ) -> Result<Option<Box<dyn crate::ResourceHolder>>, ResourcePoolError> {
        todo!()
    }
    fn deallocate(&self, holder: &dyn crate::ResourceHolder) {
        todo!()
    }
}

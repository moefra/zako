use zako_shared::FastMap;

use crate::{RequestPriority, ResourceKey, ResourceRange, shares::ResourceUnitShares};

#[derive(Clone, Debug, Default)]
pub struct ResourceAllocation<T> {
    pub items: FastMap<ResourceKey, T>,
    pub priority: RequestPriority,
}

pub type ResourceRequest = ResourceAllocation<ResourceRange>;
pub type ResourceGrant = ResourceAllocation<ResourceUnitShares>;

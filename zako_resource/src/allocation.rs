use zako_shared::FastMap;

use crate::{RequestPriority, ResourceKey, ResourceRange, shares::ResourceUnitShares};

/// A resource allocation.
#[derive(Clone, Debug, Default)]
pub struct ResourceAllocation<T> {
    pub items: FastMap<ResourceKey, T>,
    pub priority: RequestPriority,
}

/// A resource request.
pub type ResourceRequest = ResourceAllocation<ResourceRange>;

/// A resource grant.
pub type ResourceGrant = ResourceAllocation<ResourceUnitShares>;

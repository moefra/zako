//! Generic resource allocation/request containers.

use zako_shared::FastMap;

use crate::{RequestPriority, ResourceKey, ResourceRange, shares::ResourceUnitShares};

/// A resource allocation.
#[derive(Clone, Debug, Default)]
pub struct ResourceAllocation<T> {
    /// Per-key allocation payload.
    pub items: FastMap<ResourceKey, T>,
    /// Priority hint associated with the request/allocation.
    pub priority: RequestPriority,
}

/// A resource request.
pub type ResourceRequest = ResourceAllocation<ResourceRange>;

/// A resource grant.
pub type ResourceGrant = ResourceAllocation<ResourceUnitShares>;

macro_rules! res_request_item {
    () => {};
}

macro_rules! res_request {
    ( $priority:item,) => {{}};
}

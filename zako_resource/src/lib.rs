#![feature(exact_div)]

use std::fmt::Debug;

use crate::{
    allocation::{ResourceGrant, ResourceRequest},
    resource_key::ResourceKey,
    shares::ResourceUnitShares,
};

pub mod allocation;
pub mod heuristics;
pub mod pool;
pub mod resource_key;
pub mod shares;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourcePolicy {
    /// 严格限制 used + rant <= total
    Hard,
    /// 允许超卖到某个上限
    Soft { max_overcommit: ResourceUnitShares },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceDescriptor {
    pub key: ResourceKey,

    /// How many resource this descriptor stands?
    ///
    /// None for unlimited resource.
    pub total: Option<ResourceUnitShares>,

    /// The minimum allocation granularity.
    pub granularity: ResourceUnitShares,

    pub policy: ResourcePolicy,
}

impl ResourceDescriptor {
    pub fn new(
        key: ResourceKey,
        total: Option<ResourceUnitShares>,
        granularity: ResourceUnitShares,
        policy: ResourcePolicy,
    ) -> Self {
        Self {
            key,
            total,
            granularity,
            policy,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceRange {
    pub min: ResourceUnitShares,
    pub max: ResourceUnitShares, // must be >= min
}

impl ResourceRange {
    pub fn exact(v: ResourceUnitShares) -> Self {
        Self { min: v, max: v }
    }

    pub fn range(min: ResourceUnitShares, max: ResourceUnitShares) -> Option<Self> {
        if min > max {
            None
        } else {
            Some(Self { min, max })
        }
    }

    pub fn is_elastic(&self) -> bool {
        self.min != self.max
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestPriority(pub u64);

impl RequestPriority {
    pub const LOWEST: Self = Self(u64::MAX);
    pub const LOW: Self = Self(u32::MAX as u64);
    pub const NORMAL: Self = Self(u16::MAX as u64);
    pub const HIGH: Self = Self(u8::MAX as u64);
    pub const HIGHEST: Self = Self(0);
}

impl Default for RequestPriority {
    fn default() -> Self {
        Self::NORMAL
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResourcePoolError {
    #[error("unknown resource key: {0:?}")]
    UnknownResourceKey(ResourceKey),
    #[error("invalid range for {key:?}: min={min:?}, max={max:?}")]
    InvalidRange {
        key: ResourceKey,
        min: ResourceUnitShares,
        max: ResourceUnitShares,
    },
    #[error(
        "request for {key:?} is below granularity: requested={requested:?}, granularity={granularity:?}"
    )]
    BelowGranularity {
        key: ResourceKey,
        requested: ResourceUnitShares,
        granularity: ResourceUnitShares,
    },
    #[error(
        "request for {key:?} is not divisible by granularity: requested={requested:?}, granularity={granularity:?}"
    )]
    NotDivisibleByGranularity {
        key: ResourceKey,
        requested: ResourceUnitShares,
        granularity: ResourceUnitShares,
    },
    #[error("request for {key:?} exceeds hard limit: requested={requested:?}, limit={limit:?}")]
    ExceedsHardLimit {
        key: ResourceKey,
        requested: ResourceUnitShares,
        limit: ResourceUnitShares,
    },
    #[error("arithmetic overflow in resource pool")]
    ArithmeticOverflow,
    #[error("resource key mismatch with descriptor key")]
    KeyMismatch,
    #[error("other error: {0}")]
    Other(#[from] eyre::Report),
}

pub trait ResourceHolder: Debug + Send {
    fn grant(&self) -> &ResourceGrant;
    fn span(&self) -> &tracing::Span;
}

#[async_trait::async_trait]
pub trait ResourcePool: Debug + Sync + Send {
    async fn allocate(
        &self,
        request: &ResourceRequest,
    ) -> Result<Box<dyn ResourceHolder>, ResourcePoolError>;
    fn try_allocate(
        &self,
        request: &ResourceRequest,
    ) -> Result<Option<Box<dyn ResourceHolder>>, ResourcePoolError>;
    fn deallocate(&self, holder: &dyn ResourceHolder);
}

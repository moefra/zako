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

/// Resource accounting policy for a descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourcePolicy {
    /// Enforce a strict upper bound: `used + granted <= total`.
    Hard,
    /// Allow overcommit up to `total + max_overcommit`.
    Soft { max_overcommit: ResourceUnitShares },
}

/// Static metadata of one managed resource kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceDescriptor {
    /// Logical key of the resource.
    pub key: ResourceKey,

    /// Total amount represented by this descriptor.
    ///
    /// `None` means unbounded capacity.
    pub total: Option<ResourceUnitShares>,

    /// Minimum allocation granularity.
    pub granularity: ResourceUnitShares,

    /// Capacity policy for this resource.
    pub policy: ResourcePolicy,
}

impl ResourceDescriptor {
    /// Creates a new descriptor.
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

/// Requested range for one resource key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceRange {
    /// Minimum required amount.
    pub min: ResourceUnitShares,
    /// Maximum preferred amount, must be greater than or equal to `min`.
    pub max: ResourceUnitShares,
}

impl ResourceRange {
    /// Builds a non-elastic range where `min == max`.
    pub fn exact(v: ResourceUnitShares) -> Self {
        Self { min: v, max: v }
    }

    /// Builds a range and validates `min <= max`.
    pub fn range(min: ResourceUnitShares, max: ResourceUnitShares) -> Option<Self> {
        if min > max {
            None
        } else {
            Some(Self { min, max })
        }
    }

    /// Returns whether this range is elastic (`min != max`).
    pub fn is_elastic(&self) -> bool {
        self.min != self.max
    }
}

/// Priority hint attached to a resource request.
///
/// Lower numeric values represent higher priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestPriority(pub u64);

impl RequestPriority {
    /// Lowest possible priority.
    pub const LOWEST: Self = Self(u64::MAX);
    /// Low priority.
    pub const LOW: Self = Self(u32::MAX as u64);
    /// Normal priority.
    pub const NORMAL: Self = Self(u16::MAX as u64);
    /// High priority.
    pub const HIGH: Self = Self(u8::MAX as u64);
    /// Highest possible priority.
    pub const HIGHEST: Self = Self(0);
}

impl Default for RequestPriority {
    fn default() -> Self {
        Self::NORMAL
    }
}

/// Errors returned by resource-pool construction and allocation operations.
#[derive(Debug, thiserror::Error)]
pub enum ResourcePoolError {
    /// The descriptor list contains duplicate entries for the same key.
    #[error("duplicate descriptor for resource key: {0:?}")]
    DuplicateDescriptor(ResourceKey),
    /// Descriptor granularity must be non-zero.
    #[error("descriptor {key:?} has invalid zero granularity")]
    InvalidGranularity { key: ResourceKey },
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
    #[error("request for {key:?} exceeds soft limit: requested={requested:?}, limit={limit:?}")]
    ExceedsSoftLimit {
        key: ResourceKey,
        requested: ResourceUnitShares,
        limit: ResourceUnitShares,
    },
    #[error("arithmetic overflow in resource pool")]
    ArithmeticOverflow,
    #[error("internal pool state is inconsistent: {0}")]
    InconsistentState(&'static str),
    #[error("internal allocation channel was closed")]
    AllocationChannelClosed,
    #[error("resource key mismatch with descriptor key")]
    KeyMismatch,
    #[error("other error: {0}")]
    Other(#[from] eyre::Report),
}

/// A RAII handle that owns an active allocation grant.
pub trait ResourceHolder: Debug + Send {
    /// Returns the unique allocation identity in the owning pool.
    fn allocation_id(&self) -> u64;
    /// Returns the granted resources represented by this holder.
    fn grant(&self) -> &ResourceGrant;
    /// Returns the tracing span associated with this holder.
    fn span(&self) -> &tracing::Span;
}

/// A resource-pool implementation that supports eager and waiting allocations.
#[async_trait::async_trait]
pub trait ResourcePool: Debug + Sync + Send {
    /// Allocates resources for `request`.
    ///
    /// This method may wait until the request becomes allocatable.
    /// Structural validation and arithmetic errors are returned immediately.
    async fn allocate(
        &self,
        request: &ResourceRequest,
    ) -> Result<Box<dyn ResourceHolder>, ResourcePoolError>;
    /// Attempts to allocate resources for `request` without waiting.
    ///
    /// Returns:
    /// - `Err(...)` for structural validation and arithmetic errors.
    /// - `Ok(None)` only when the request is valid but currently not satisfiable.
    /// - `Ok(Some(holder))` when allocation succeeds immediately.
    fn try_allocate(
        &self,
        request: &ResourceRequest,
    ) -> Result<Option<Box<dyn ResourceHolder>>, ResourcePoolError>;
    /// Releases resources held by `holder`.
    ///
    /// Releasing an unknown or already-released allocation is a no-op.
    fn deallocate(&self, holder: &dyn ResourceHolder);
}

#![feature(exact_div)]

use crate::shares::ResourceUnitShares;

pub mod allocation;
pub mod heuristics;
pub mod pool;
pub mod shares;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKey {
    Processor,
    Memory,
    Disk,
    Network,
    Gpu,
    Other(zako_id::UniqueId),
}

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

use smol_str::SmolStr;

pub mod heuristics;

pub type ResourceUnit = u64;

pub static RESOURCE_UNIT_MULTIPLIER: ResourceUnit = 1_000_000;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OverflowError(());

impl Default for OverflowError {
    fn default() -> Self {
        Self(())
    }
}

impl std::fmt::Display for OverflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "out of range integral type operation attempted".fmt(f)
    }
}

impl std::error::Error for OverflowError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceUnitShares(ResourceUnit);

impl TryFrom<ResourceUnit> for ResourceUnitShares {
    type Error = OverflowError;

    fn try_from(value: ResourceUnit) -> Result<Self, Self::Error> {
        value
            .checked_mul(RESOURCE_UNIT_MULTIPLIER)
            .map(|v| Self(v))
            .ok_or(Default::default())
    }
}

pub enum ResourceKey {
    Processor,
    Memory,
    Disk,
    Network,
    Gpu,
    Other(SmolStr),
}

pub struct ResourceDescriptor {
    key: ResourceKey,
    total: ResourceUnitShares,
}

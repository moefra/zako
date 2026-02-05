pub type ResourceUnit = u64;

pub type FloatResourceUnit = f64;

pub static RESOURCE_UNIT_MULTIPLIER: ResourceUnit = 1_000_000;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ResourceOverflowError(());

impl Default for ResourceOverflowError {
    fn default() -> Self {
        Self(())
    }
}

impl std::fmt::Display for ResourceOverflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "out of range integral type operation attempted".fmt(f)
    }
}

impl std::error::Error for ResourceOverflowError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceUnitShares(ResourceUnit);

impl TryFrom<ResourceUnit> for ResourceUnitShares {
    type Error = ResourceOverflowError;

    fn try_from(value: ResourceUnit) -> Result<Self, Self::Error> {
        value
            .checked_mul(RESOURCE_UNIT_MULTIPLIER)
            .map(|v| Self(v))
            .ok_or(Default::default())
    }
}

impl ResourceUnitShares {
    pub fn from_unit(value: ResourceUnit) -> Self {
        Self(value * RESOURCE_UNIT_MULTIPLIER)
    }

    pub fn from_shares(value: ResourceUnit) -> Self {
        Self(value)
    }

    pub fn as_shares(&self) -> ResourceUnit {
        self.0
    }

    pub fn as_unit(&self) -> FloatResourceUnit {
        (self.0 as FloatResourceUnit) / (RESOURCE_UNIT_MULTIPLIER as FloatResourceUnit)
    }
}

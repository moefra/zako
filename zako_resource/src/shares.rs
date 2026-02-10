use thiserror::Error;

pub type ResourceUnit = u64;

pub static RESOURCE_UNIT_MULTIPLIER: ResourceUnit = 1_000_000;

#[derive(Debug, Clone, Error)]
pub enum ResourceUnitComputionError {
    #[error("out of range resource unit operation attempted")]
    OverflowError(),
    #[error("the div operation on resource unit or shares has reminder")]
    HasReminder(),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceUnitShares(ResourceUnit);

impl TryFrom<ResourceUnit> for ResourceUnitShares {
    type Error = ResourceUnitComputionError;

    fn try_from(value: ResourceUnit) -> Result<Self, Self::Error> {
        value
            .checked_mul(RESOURCE_UNIT_MULTIPLIER)
            .map(Self)
            .ok_or(ResourceUnitComputionError::OverflowError())
    }
}

impl ResourceUnitShares {
    pub fn from_shares(value: ResourceUnit) -> Self {
        Self(value)
    }

    pub fn as_shares(&self) -> ResourceUnit {
        self.0
    }

    pub fn try_as_unit(&self) -> Result<ResourceUnit, ResourceUnitComputionError> {
        self.0
            .div_exact(RESOURCE_UNIT_MULTIPLIER)
            .ok_or(ResourceUnitComputionError::HasReminder())
    }

    pub fn checked_add(&self, rhs: ResourceUnitShares) -> Option<ResourceUnitShares> {
        let lhs = *self;
        Some(ResourceUnitShares::from_shares(
            lhs.as_shares().checked_add(rhs.as_shares())?,
        ))
    }

    pub fn checked_sub(&self, rhs: ResourceUnitShares) -> Option<ResourceUnitShares> {
        let lhs = *self;
        Some(ResourceUnitShares::from_shares(
            lhs.as_shares().checked_sub(rhs.as_shares())?,
        ))
    }
}

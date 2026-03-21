//! Utility types for fixed-point resource accounting in share units.

use thiserror::Error;

/// Base integer type used for all resource-unit math.
pub type ResourceUnit = u64;

/// Fixed-point multiplier from units to shares.
pub static RESOURCE_UNIT_MULTIPLIER: ResourceUnit = 1_000_000;

/// Errors produced by share/unit conversion and arithmetic helpers.
#[derive(Debug, Clone, Error)]
pub enum ResourceUnitComputionError {
    /// Arithmetic overflow occurred.
    #[error("out of range resource unit operation attempted")]
    OverflowError(),
    /// Division did not produce an exact result.
    #[error("the div operation on resource unit or shares has reminder")]
    HasReminder(),
}

/// Fixed-point resource amount represented in share units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceUnitShares(ResourceUnit);

impl TryFrom<ResourceUnit> for ResourceUnitShares {
    type Error = ResourceUnitComputionError;

    /// Converts whole units into shares by multiplying with the fixed multiplier.
    fn try_from(value: ResourceUnit) -> Result<Self, Self::Error> {
        value
            .checked_mul(RESOURCE_UNIT_MULTIPLIER)
            .map(Self)
            .ok_or(ResourceUnitComputionError::OverflowError())
    }
}

impl ResourceUnitShares {
    /// Creates a value directly from raw share units.
    pub fn from_shares(value: ResourceUnit) -> Self {
        Self(value)
    }

    /// Returns the raw share count.
    pub fn as_shares(&self) -> ResourceUnit {
        self.0
    }

    /// Converts shares into whole units only when divisible by the multiplier.
    pub fn try_as_unit(&self) -> Result<ResourceUnit, ResourceUnitComputionError> {
        self.0
            .div_exact(RESOURCE_UNIT_MULTIPLIER)
            .ok_or(ResourceUnitComputionError::HasReminder())
    }

    /// Checked addition in share units.
    pub fn checked_add(&self, rhs: ResourceUnitShares) -> Option<ResourceUnitShares> {
        let lhs = *self;
        Some(ResourceUnitShares::from_shares(
            lhs.as_shares().checked_add(rhs.as_shares())?,
        ))
    }

    /// Checked subtraction in share units.
    pub fn checked_sub(&self, rhs: ResourceUnitShares) -> Option<ResourceUnitShares> {
        let lhs = *self;
        Some(ResourceUnitShares::from_shares(
            lhs.as_shares().checked_sub(rhs.as_shares())?,
        ))
    }
}

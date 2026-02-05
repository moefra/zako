use smol_str::SmolStr;

use crate::shares::ResourceUnitShares;

pub mod heuristics;
pub mod shares;

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
    total: Option<ResourceUnitShares>,
}

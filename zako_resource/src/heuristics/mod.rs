use sysinfo::System;

use crate::shares::ResourceUnitShares;

pub fn memory(system: &System) -> ResourceUnitShares {
    ResourceUnitShares::from_shares(system.total_memory())
}

pub fn cpu(system: &System) -> ResourceUnitShares {
    ResourceUnitShares::from_shares(system.cpus().len() as u64)
}

pub fn disk(system: &System) -> ResourceUnitShares {
    ResourceUnitShares::from_shares(system.cpus().len() as u64)
}

pub fn network(system: &System) -> ResourceUnitShares {
    ResourceUnitShares::from_shares(system.cpus().len() as u64)
}

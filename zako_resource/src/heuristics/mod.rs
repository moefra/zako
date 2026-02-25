use sysinfo::System;

use crate::shares::ResourceUnitShares;

pub fn total_memory_capacity(system: &System) -> ResourceUnitShares {
    ResourceUnitShares::from_shares(system.total_memory())
}

pub fn cpu_thread_count(system: &System) -> ResourceUnitShares {
    ResourceUnitShares::from_shares(system.cpus().len() as u64)
}

pub fn disk_free_space(system: &System) -> ResourceUnitShares {
    ResourceUnitShares::from_shares(system.cpus().len() as u64)
}

pub fn network_bandwidth(system: &System) -> ResourceUnitShares {
    ResourceUnitShares::from_shares(system.cpus().len() as u64)
}

//! Host-derived heuristic helpers for default resource descriptors.

use sysinfo::System;

use crate::shares::ResourceUnitShares;

/// Returns total host memory as share units.
pub fn memory_capacity_in_byte(system: &System) -> ResourceUnitShares {
    ResourceUnitShares::from_shares(system.total_memory())
}

/// Returns logical CPU thread count as share units.
pub fn cpu_thread_count(system: &System) -> ResourceUnitShares {
    ResourceUnitShares::from_shares(system.cpus().len() as u64)
}

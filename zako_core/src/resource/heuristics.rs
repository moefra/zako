use std::time::Duration;

use sysinfo::System;

pub fn determine_memory_cache_size_for_cas(system: &System) -> u64 {
    let total_ram = system.total_memory();

    let target = total_ram / 10; // 10%

    target.clamp(4 * 1024 * 1024, 4 * 1024 * 1024 * 1024) // 4MB ~ 4GB
}

pub fn determine_memory_ttl_for_cas(system: &System) -> Duration {
    Duration::from_secs(60 * 30) // 30minutes
}

pub fn determine_memory_tti_for_cas(system: &System) -> Duration {
    Duration::from_secs(5 * 60) // 5 minutes
}

pub fn determine_oxc_workers_count(system: &System) -> usize {
    1
}

pub fn determine_v8_workers_count(system: &System) -> usize {
    usize::try_from(system.cpus().len() / 2).unwrap_or(1)
}

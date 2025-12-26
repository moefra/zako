use ::std::{path::PathBuf, time::Duration};

use sysinfo::System;

mod std;

pub use std::generate_platform_configuration;

use crate::worker::worker_pool::PoolConfig;

pub fn determine_memory_cache_size_for_cas(system: &System) -> u64 {
    let total_ram = system.total_memory();

    let target = total_ram / 10; // 10%

    target.clamp(4 * 1024 * 1024, 4 * 1024 * 1024 * 1024) // 4MB ~ 4GB
}

pub fn determine_memory_ttl_for_cas(_system: &System) -> Duration {
    Duration::from_secs(60 * 30) // 30minutes
}

pub fn determine_memory_tti_for_cas(_system: &System) -> Duration {
    Duration::from_secs(5 * 60) // 5 minutes
}

pub fn determine_oxc_workers_count(_system: &System) -> usize {
    1
}

pub fn determine_v8_workers_count(system: &System) -> usize {
    usize::try_from(system.cpus().len() / 2).unwrap_or(1)
}

pub fn determine_oxc_workers_config(system: &System) -> PoolConfig {
    PoolConfig {
        max_workers: determine_oxc_workers_count(system),
        min_workers: 1,
        idle_timeout: Duration::from_secs(60),
    }
}

pub fn determine_v8_workers_config(system: &System) -> PoolConfig {
    PoolConfig {
        max_workers: determine_v8_workers_count(system),
        min_workers: 1,
        idle_timeout: Duration::from_secs(60),
    }
}

pub fn determine_tokio_thread_stack_size(_: &System) -> usize {
    // 4MB
    1024 * 1024 * 4
}

pub fn determine_local_cas_path(_: &System) -> PathBuf {
    PathBuf::from(format!(
        "{}/{}/{}",
        ::dirs::cache_dir()
            .unwrap_or(PathBuf::from("~/"))
            .to_string_lossy(),
        "zako",
        "cas"
    ))
}

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
    usize::try_from(system.cpus().len() / 2).unwrap_or(1).min(1)
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

/// Determines the CPU capacity for the resource pool.
/// Returns the number of logical CPU cores.
pub fn determine_cpu_capacity(system: &System) -> u64 {
    system.cpus().len() as u64
}

/// Determines the memory capacity for the resource pool.
/// Returns total system memory in bytes.
pub fn determine_memory_capacity(system: &System) -> u64 {
    system.total_memory()
}

/// Determines the disk IO capacity for the resource pool.
/// Returns a heuristic value based on available parallelism.
pub fn determine_disk_io_capacity(system: &System) -> u64 {
    // Use 2x CPU count as a reasonable default for concurrent IO operations
    (system.cpus().len() * 2).max(4) as u64
}

/// Determines the network capacity for the resource pool.
/// Returns a heuristic value for concurrent network connections.
pub fn determine_network_capacity(system: &System) -> u64 {
    // Use 4x CPU count as a reasonable default for concurrent network operations
    (system.cpus().len() * 4).max(8) as u64
}

/// Determines the GPU capacity for the resource pool.
/// Returns 0 if no GPU detection is available, otherwise a reasonable default.
pub fn determine_gpu_capacity(_system: &System) -> u64 {
    // sysinfo doesn't provide GPU info directly; return 0 as default
    // Users can override this with explicit values
    0
}

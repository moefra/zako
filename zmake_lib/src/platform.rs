use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, OnceLock};

static PLATFORM:OnceLock<v8::SharedRef<v8::Platform>> = OnceLock::new();

pub fn get_initialized_or_default() -> v8::SharedRef<v8::Platform> {
    initialize(num_cpus::get() as u32, false)
}

pub fn initialize(thread_pool_size: u32, idle_task_support: bool) -> v8::SharedRef<v8::Platform>{
    PLATFORM.get_or_init(|| {
        let platform = v8::new_default_platform(thread_pool_size, idle_task_support).make_shared();
        v8::V8::initialize_platform(platform.clone());
        v8::V8::initialize();
        platform
    }).clone()
}

use deno_core::v8;
use std::sync::OnceLock;
use tracing::error;

static PLATFORM: OnceLock<v8::SharedRef<v8::Platform>> = OnceLock::new();

pub fn get_set_platform_or_default() -> v8::SharedRef<v8::Platform> {
    set_platform(num_cpus::get() as u32 + 1, false)
}

pub fn set_platform(thread_pool_size: u32, idle_task_support: bool) -> v8::SharedRef<v8::Platform> {
    let platform = PLATFORM
        .get_or_init(|| {
            let platform =
                v8::new_default_platform(thread_pool_size, idle_task_support).make_shared();
            platform
        })
        .clone();
    v8::V8::set_fatal_error_handler(|file: &'_ str, line, message: &'_ str| {
        error!(
            "Get a v8 fatal error: at file `{}` line `{}`,reason:{}",
            file, line, message
        )
    });
    platform
}

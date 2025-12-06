deno_core::extension!(
    zako_core,
    deps = [zako_global, zako_syscall, zako_semver],
    esm_entry_point = "zako:core",
    esm = ["zako:core" = "builtins/core.js"],
    docs = "The extension that provide necessary core for zako",
);

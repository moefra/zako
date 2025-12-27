deno_core::extension!(
    zako_config,
    deps = [zako_syscall],
    esm_entry_point = "zako:config",
    esm = ["zako:config" = "../dist/builtins/config.js"],
    docs = "The extension that provide necessary config for zako",
);

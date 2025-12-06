deno_core::extension!(
    zako_console,
    deps = [zako_global, zako_syscall, zako_semver],
    esm_entry_point = "zako:console",
    esm = ["zako:console" = "builtins/console.js"],
    docs = "The extension that provide console for zako",
);

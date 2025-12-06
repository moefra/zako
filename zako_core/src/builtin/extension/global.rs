deno_core::extension!(
    zako_global,
    deps = [zako_syscall],
    esm_entry_point = "zako:global",
    esm = ["zako:global" = "../dist/builtins/global.js"],
    docs = "The extension that provide necessary global for zako",
);

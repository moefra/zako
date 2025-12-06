deno_core::extension!(
    zako_rt,
    esm_entry_point = "zako:rt",
    esm = ["zako:rt" = "builtins/rt.js"],
    docs = "The extension that provide necessary setup for zako",
);

deno_core::extension!(
    zako_rt,
    esm_entry_point = "zako:semver",
    esm = ["zako:rt" = "../dist/builtins/rt.js"],
    docs = "The extension that provide necessary setup for zako",
);

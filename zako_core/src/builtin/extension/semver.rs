deno_core::extension!(
    zako_semver,
    deps = [zako_rt],
    esm_entry_point = "zako:semver",
    esm = ["zako:semver" = "builtins/semver.js"],
    docs = "The extension that provide node-semver for zako",
);

deno_core::extension!(
    zako_rt,
    esm_entry_point = "zako:semver",
    esm = ["zako:rt" = "../dist/builtins/project.js"],
    docs = "The extension that provide project related APIs for zako",
);

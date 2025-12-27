deno_core::extension!(
    zako_project,
    deps = [zako_core],
    esm_entry_point = "zako:project",
    esm = ["zako:project" = "../dist/builtins/project.js"],
    docs = "The extension that provide project related APIs for zako",
);

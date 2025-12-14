import { ccLibrary, ccBinary } from "zako:rule";
import { project } from "zako:project";
const lib = ccLibrary({
    name: "benchmark_lib",
    srcs: ["src/**/*.cpp"],
});
const bin = project.ccBinary({
    name: "benchmark_bin",
    srcs: ["app/main.cpp"],
    deps: [lib],
});
if (project.config["guava-cpp20"]) {
    lib.cxxStandard = 20;
    bin.cxxStandard = 20;
}
if (project.config.log) {
    lib.links.push("//logging");
    bin.links.push("//logging");
}
export default [lib, bin];

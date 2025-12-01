import { defineConfig } from "tsdown";

let base = {
    dts: {
        sourcemap: true,
    },
    fixedExtension: true,
    minify: true,
    platform: "neutral",
    target: "esnext",
    tsconfig: "./tsconfig.json",
    format: "es",
    clean: true,
    hash: false,
    shims: false,
    external: ["zmake:syscall", "zmake:semver"],
};

export default defineConfig([
    {
        ...base,
        entry: "semver/index.js",
        outDir: "dist/semver",
        copy: [{ from: "semver.jsr.json", to: "dist/semver/jsr.json" }],
    },
    {
        ...base,
        entry: "rt.ts",
        outDir: "dist/rt",
    },
    {
        ...base,
        entry: "core.ts",
        outDir: "dist/core",
    },
]);

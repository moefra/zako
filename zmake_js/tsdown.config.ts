import { defineConfig } from "tsdown";

let base = {
    dts: {
        sourcemap: true,
    },
    fixedExtension: true,
    minify: true,
    platform: "neutral",
    target: "es2023",
    tsconfig: "tsconfig.json",
    format: "es",
};

export default defineConfig([
    {
        entry: "semver.ts",
        outDir: "dist/semver",
        ...base,
    },
    {
        entry: "rt.ts",
        outDir: "dist/rt",
        ...base,
    },
]);

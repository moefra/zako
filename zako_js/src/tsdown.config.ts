import { defineConfig } from "tsdown";

let current = import.meta.dirname;

if(current == undefined){
    current = Deno.cwd();
}

const dist = `${current}/../../dist/`;

let base = {
    dts: {
        sourcemap: false,
    },
    fixedExtension: true,
    minify: true,
    platform: `neutral`,
    target: `esnext`,
    tsconfig: `${current}/tsconfig.json`,
    format: `es`,
    clean: true,
    hash: false,
    shims: false,
    external: [/^zmake/],
};

export default defineConfig([
    {
        ...base,
        entry: `${current}/../semver/index.js`,
        outDir: `${dist}/semver`,
        copy: [{ from: `${current}/semver.jsr.json`, to: `${dist}/semver/jsr.json` }],
    },
    {
        ...base,
        entry: `${current}/rt.ts`,
        outDir: `${dist}/rt`,
    },
    {
        ...base,
        entry: `${current}/core.ts`,
        outDir: `${dist}/core`,
    },
]);

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
    external: [/^zako/],
};

export default defineConfig([
    {
        ...base,
        entry: `${current}/../semver/index.js`,
        outDir: `${dist}/semver`,
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
    {
        ...base,
        entry: `${current}/global.ts`,
        outDir: `${dist}/global`,
    },
    {
        ...base,
        entry: `${current}/console.ts`,
        outDir: `${dist}/console`,
    },
    {
        ...base,
        entry: `${current}/project.ts`,
        outDir: `${dist}/project`,
    },
]);

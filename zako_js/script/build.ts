
import * as fs from "node:fs/promises";

const current = import.meta.dir;
const dist = `${current}/../../dist/`;
const src = `${current}/../src/`;
const builtins_src = `${src}/builtins/`;
const builtins_dist = `${dist}/builtins/`;

// transpile builtins/*.ts into dist/builtins/*.js
console.log("transpile...");
{
    const transpiler = new Bun.Transpiler({
        loader: "ts",
        treeShaking: false,
        allowBunRuntime: false,
        trimUnusedImports: false,
        target: "browser",
        tsconfig: await fs.readFile(`${src}/tsconfig.json`, "utf-8"), // TODO:Process the path of extends `../tsconfig.json`
    });

    let transformTasks = [];
    const dirs = await fs.readdir(`${builtins_src}`,
        { withFileTypes: true,
        recursive:false });

    await fs.mkdir(
    `${builtins_dist}`,
    { recursive: true }
    );

    for (let dir of dirs){
        if(dir.isFile()){
            continue;
        }

        const entry = `${builtins_src}/${dir.name}/index.ts`;
        const out = `${builtins_dist}/${dir.name}.js`;

        console.log(`transpile ${entry} -> ${out}`);

        transformTasks.push(
            fs.readFile(entry, "utf-8").then(async (source) =>
            transpiler.transform(source).then(async result => {
                return await fs.writeFile(out, result);
            })
        ));
    }

    for(let task of transformTasks){
        await task;
    }
}

// bundle semver into one file
console.log("bundle source files...");
{
    console.log(`bundle semver into dist/builtins/semver.js`);
    await Bun.build({
        entrypoints: [`${builtins_src}/semver/index.ts`],
        outdir: `${builtins_dist}`,
        minify: false,
        sourcemap: "inline",
        target: 'browser',
        splitting: false,
        format: 'esm',
        external: ["zako:*"],
        packages: "bundle",
        naming: "[dir]/semver.[ext]", // for semver only
    });
}

// bundle types for semver
console.log("bundle type declaration files...");
{
    const proc = Bun.spawn(["bun", "x", "dts-bundle-generator",
            "--project",
            `${src}/tsconfig.json`,
            "--external-inlines=@types/semver",
            "-o",
            `${dist}/semver/index.d.ts`,
            "--",
            `${current}/../node_modules/@types/semver/index.d.ts`,], {
        cwd: dist,
        stderr: "inherit",
        stdout: "inherit",
        stdin: "ignore",
    });

    await proc.exited;
}

// collect xxx/index.d.ts into types/*.d.ts
console.log("simplify directory structure...");
{
    await fs.mkdir(`${dist}/types`, { recursive: true });

    const typeDirs = await fs.readdir(`${builtins_dist}/`,
        { withFileTypes: true, recursive:false });

    for (let dir of typeDirs){
        if(dir.isFile()){
            continue;
        }

        const entry = `${builtins_dist}/${dir.name}/index.d.ts`;
        const out = `${dist}/types/${dir.name}.d.ts`;

        console.log(`copy type ${entry} -> ${out}`);

        await fs.copyFile(entry, out);
        // 过河拆桥
        await fs.rm(`${builtins_dist}/${dir.name}`, { recursive: true, force: true });
    }
}

// remove dist/xxx directory, left dist/builtins/ and dist/types/
console.log("remove needless directory...");
{
    let dirs = await fs.readdir(`${dist}/`,
    { withFileTypes: true, recursive:false });

    for (let dir of dirs){
        if(dir.isFile() || dir.name === "builtins" || dir.name === "types"){
            continue;
        }

        const target = `${dist}/${dir.name}`;
        console.log(`remove directory ${target}`);
        await fs.rm(target, { recursive: true, force: true });
    }
}

let collectedBuiltinCommonModules: string[] = [];
console.log("mv project/....d.ts to .../....d.ts and make file references to global...");
{
    let modules = await fs.readdir(`${dist}/types/`);
    for (let mod of modules){
        if(!mod.endsWith(".d.ts")){
            continue;
        }

        const modName = mod.substring(0, mod.length - 5); // remove .d.ts
        if(modName.startsWith("index")){
            continue;
        }
        console.log(`export module zako:${modName}`);

        // write /// <reference types="zako:global" />
        let globalPrefix = "./";
        if(modName === "project" || modName === "build" || modName === "rule" || modName === "toolchain"){
            globalPrefix = "../";
        }
        await fs.writeFile(
            `${dist}/types/${mod}`,
            `/// <reference path="${globalPrefix}global.d.ts" />\n` +
            await fs.readFile(`${dist}/types/${mod}`),
        "utf-8");

        collectedBuiltinCommonModules.push(`zako:${modName}`);

        // move project/build/... .d.ts to project/project.d.ts etc.
        await fs.mkdir(`${dist}/types/project`, { recursive: true });
        await fs.mkdir(`${dist}/types/project`, { recursive: true });
        await fs.mkdir(`${dist}/types/build`, { recursive: true });
        await fs.mkdir(`${dist}/types/rule`, { recursive: true });
        await fs.mkdir(`${dist}/types/toolchain`, { recursive: true });
        if(modName === "project"){
            await fs.rename(
                `${dist}/types/${mod}`,
                `${dist}/types/project/${mod}`
            );
        }
        else if(modName === "build"){
            await fs.rename(
                `${dist}/types/${mod}`,
                `${dist}/types/build/${mod}`
            );
        }
        else if(modName === "rule"){
            await fs.rename(
                `${dist}/types/${mod}`,
                `${dist}/types/rule/${mod}`
            );
        }
        else if(modName === "toolchain"){
            await fs.rename(
                `${dist}/types/${mod}`,
                `${dist}/types/toolchain/${mod}`
            );
        }
    }
}

// copy package.json,README.md,LICENSE into dist/
console.log("copy package.json, README.md, LICENSE to dist/types...");
{
    await fs.copyFile(
        `${current}/../package.json`,
        `${dist}/types/package.json`
    );
    await fs.copyFile(
        `${current}/../../README.md`,
        `${dist}/types/README.md`
    );
    await fs.copyFile(
        `${current}/../../LICENSE`,
        `${dist}/types/LICENSE`
    );
}

// generate template directory
console.log("generate template...")
{
    await fs.mkdir(`${dist}/template`, { recursive: true });
}

// generate template tsconfig.json
import * as JSON5 from "json5";

function createConfig(name:string, includes:string[], enableType?:string, extraExcludes:string[] = [], refs:string[]  = []):any {
    let config = {
        extends: "./tsconfig.base.json",
        compilerOptions: {
            paths: {
                "zako:*": ["./.zako/types/zako/*.d.ts"],
            },
            typeRoots: ["./.zako/types/", "./node_modules/@types/"],
            outDir: `./dist/${name}`,
        },
        include: includes,
        exclude: [
            "node_modules",
            "dist",
            ".zako",
            ...extraExcludes
        ],
        references: refs.map(path => ({ path }))
    };
    if(enableType != undefined){
        (config.compilerOptions as any).paths[`zako:${enableType}`] = [`./.zako/types/zako/${enableType}/${enableType}.d.ts`];
    }
    return config;
};

console.log("generate template/tsconfig.json...")
{
    // best js/ts practice included!
    const baseConfig = JSON5.parse(await fs.readFile(`${current}/../tsconfig.json`, "utf-8")) as any;

    delete baseConfig["reference"];
    delete baseConfig["include"];
    delete baseConfig["exclude"];
    baseConfig["files"] = [];
    baseConfig.compilerOptions.composite = true;
    baseConfig.compilerOptions.declarationMap = true;

    const rootConfig = {
        files: [],
        references: [
            { path: "./tsconfig.library.json" },
            { path: "./tsconfig.project.json" },
            { path: "./tsconfig.build.json" },
            { path: "./tsconfig.rule.json" },
            { path: "./tsconfig.toolchain.json" },
            { path: "./tsconfig.script.json" }
        ]
    };

    const patterns = {
        project: ["./**/zako.ts"],
        build: ["./**/BUILD.ts"],
        rule: ["./**/*.rule.ts"],
        toolchain: ["./**/*.toolchain.ts"],
        script: ["./**/*.script.ts", "./**/scripts/**/*.ts"],
        library: ["./**/*.ts"]
    };

    const libraryConfig = createConfig(
    "library",
    patterns.library,
    undefined,
    [
        ...patterns.project,
        ...patterns.build,
        ...patterns.rule,
        ...patterns.toolchain,
        ...patterns.script
    ],
    []
    );

    const commonRefs = ["./tsconfig.library.json"];

    const projectConfig = createConfig(
        "project",
        patterns.project,
    "project",
        [],
        commonRefs
    );

    const buildConfig = createConfig(
        "build",
        patterns.build,
        "build",
        [],
        [...commonRefs, "./tsconfig.rule.json"] // Build 可能引用 Rule
    );

    const ruleConfig = createConfig(
        "rule",
        patterns.rule,
        "rule",
        [],
        commonRefs
    );

    const toolchainConfig = createConfig(
        "toolchain",
        patterns.toolchain,
        "toolchain",
        [],
        commonRefs
    );

    let scriptConfig = createConfig(
        "script",
        patterns.script,
        undefined,
        [],
        commonRefs
    );
    delete scriptConfig.compilerOptions.paths;
    scriptConfig.compilerOptions["types"] = ["node"];

    await fs.writeFile(
        `${dist}/template/tsconfig.json`,
        JSON.stringify(rootConfig, null, 4),
        "utf-8",
    );
    await fs.writeFile(
        `${dist}/template/tsconfig.base.json`,
        JSON.stringify(baseConfig, null, 4),
        "utf-8",
    );
    await fs.writeFile(
        `${dist}/template/tsconfig.build.json`,
        JSON.stringify(buildConfig, null, 4),
        "utf-8"
    );
    await fs.writeFile(
        `${dist}/template/tsconfig.project.json`,
        JSON.stringify(projectConfig, null, 4),
        "utf-8"
    );
    await fs.writeFile(
        `${dist}/template/tsconfig.rule.json`,
        JSON.stringify(ruleConfig, null, 4),
        "utf-8"
    );
    await fs.writeFile(
        `${dist}/template/tsconfig.toolchain.json`,
        JSON.stringify(toolchainConfig, null, 4),
        "utf-8"
    );
    await fs.writeFile(
        `${dist}/template/tsconfig.library.json`,
        JSON.stringify(libraryConfig, null, 4),
        "utf-8"
    );
    await fs.writeFile(
        `${dist}/template/tsconfig.script.json`,
        JSON.stringify(scriptConfig, null, 4),
        "utf-8"
    );
}

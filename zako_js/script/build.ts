
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

// generate index.d.ts
let collectedBuiltinCommonModules: string[] = [];
console.log("generate index.d.ts...");
{
    let commonSource = `export {};\n`;

    let projectSource = `${commonSource}`;
    let buildSource = `${commonSource}`;
    let ruleSource = `${commonSource}`;
    let toolchainSource = `${commonSource}`;

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

        let modSource = "";
        if(modName != "console"){
            modSource += `import type {} from "./${modName}";\n`;
        }
        else{
            modSource += `import type { Console } from "./${modName}";\n`;
        }
        modSource += `declare module "zako:${modName}" {\n`;
        modSource += `    export * from "./${modName}";\n`;
        modSource += `}\n`;

        if (modName == "project") {
            projectSource += modSource.replaceAll("./","../");
        } else if (modName == "build") {
            buildSource += modSource.replaceAll("./","../");
        } else if (modName == "rule") {
            ruleSource += modSource.replaceAll("./","../");
        } else if (modName == "toolchain") {
            toolchainSource += modSource.replaceAll("./","../");
        } else {
            commonSource += modSource;
            projectSource += modSource.replaceAll("./","../");
            buildSource += modSource.replaceAll("./","../");
            ruleSource += modSource.replaceAll("./","../");
            toolchainSource += modSource.replaceAll("./","../");
            collectedBuiltinCommonModules.push(`zako:${modName}`);
        }
    }

    commonSource += `declare global { export const console: Console; }\n`
    projectSource += `declare global { export const console: Console; }\n`
    buildSource += `declare global { export const console: Console; }\n`
    ruleSource += `declare global { export const console: Console; }\n`
    toolchainSource += `declare global { export const console: Console; }\n`

    await fs.mkdir(`${dist}/types/project/`, { recursive: true });
    await fs.mkdir(`${dist}/types/build/`, { recursive: true });
    await fs.mkdir(`${dist}/types/rule/`, { recursive: true });
    await fs.mkdir(`${dist}/types/toolchain/`, { recursive: true });
    await fs.writeFile(
        `${dist}/types/index.d.ts`,
        commonSource,
        "utf-8"
    );
    await fs.writeFile(
        `${dist}/types/project/index.d.ts`,
        projectSource,
        "utf-8"
    );
    await fs.writeFile(
        `${dist}/types/build/index.d.ts`,
        buildSource,
        "utf-8"
    );
    await fs.writeFile(
        `${dist}/types/rule/index.d.ts`,
        ruleSource,
        "utf-8"
    );
    await fs.writeFile(
        `${dist}/types/toolchain/index.d.ts`,
        toolchainSource,
        "utf-8"
    );
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

function createConfig(name:string, includes:string[], types:string[], extraExcludes:string[] = [], refs:string[]  = []):any {
    return {
        extends: "./tsconfig.base.json",
        compilerOptions: {
            types: types,
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
    ["zako"],
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
    ["zako/project"],
        [],
        commonRefs
    );

    const buildConfig = createConfig(
        "build",
        patterns.build,
        ["zako/build"],
        [],
        [...commonRefs, "./tsconfig.rule.json"] // Build 可能引用 Rule
    );

    const ruleConfig = createConfig(
        "rule",
        patterns.rule,
        ["zako/rule"],
        [],
        commonRefs
    );

    const toolchainConfig = createConfig(
        "toolchain",
        patterns.toolchain,
        ["zako/toolchain"],
        [],
        commonRefs
    );

    const scriptConfig = createConfig(
        "script",
        patterns.script,
        ["node"],
        [],
        commonRefs
    );

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

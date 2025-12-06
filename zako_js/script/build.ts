
const current = import.meta.dir;
const dist = `${current}/../dist/`;
const src = `${current}/../src/`;
const builtins = `${src}/builtins/`;

import * as fs from "node:fs/promises";

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
    const dirs = await fs.readdir(`${builtins}`,
        { withFileTypes: true,
        recursive:false });

    await fs.mkdir(
    `${dist}/builtins/`,
    { recursive: true }
    );

    for (let dir of dirs){
        if(dir.isFile()){
            continue;
        }

        const entry = `${builtins}/${dir.name}/index.ts`;
        const out = `${dist}/builtins/${dir.name}.js`;

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
        entrypoints: [`${builtins}/semver/index.ts`],
        outdir: `${dist}/builtins/`,
        minify: true,
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
            //"--project",
            //`${src}/tsconfig.json`,
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

    const typeDirs = await fs.readdir(`${dist}/`,
        { withFileTypes: true, recursive:false });

    for (let dir of typeDirs){
        if(dir.isFile()){
            continue;
        }

        if(dir.name === "builtins" || dir.name === "types"){
            continue;
        }

        const entry = `${dist}/${dir.name}/index.d.ts`;
        const out = `${dist}/types/${dir.name}.d.ts`;

        console.log(`copy type ${entry} -> ${out}`);

        await fs.copyFile(entry, out);
    }
}

// remove dist/xxx directory, left dist/builtins/ and dist/types/
console.log("remove needless directory...");
{
    const dirs = await fs.readdir(`${dist}/`,
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
console.log("generate index.d.ts...");
{
    let source = `/// <reference no-default-lib="true"/>\n`;
    source += `/// <reference lib="esnext"/>\n`; // TODO: use es2025 when ts-go release
    source += `export {};\n`;

    let modules = await fs.readdir(`${dist}/types/`);
    for (let mod of modules){
        if(!mod.endsWith(".d.ts")){
            continue;
        }
        const modName = mod.substring(0, mod.length - 5); // remove .d.ts
        if(modName != "console"){
            source += `import type {} from "./${modName}";\n`;
        }
        else{
            source += `import type { Console } from "./${modName}";\n`;
        }
        source += `declare module "zako:${modName}" {\n`;
        source += `    export * from "./${modName}";\n`;
        source += `}\n`;
    }

    source += `declare global { export const console: Console; }\n`

    await fs.writeFile(
        `${dist}/types/index.d.ts`,
        source,
        "utf-8"
    );
}

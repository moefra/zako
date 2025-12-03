import { build } from 'tsdown'

let current = import.meta.dirname;

if(current == undefined){
    current = Deno.cwd();
}

await build({
    config: `${current}/tsdown.config.ts`,
});

let command = new Deno.Command(Deno.execPath(), {
    args: [
        "run",
        "--allow-all",
        "dts-bundle-generator",
        "--project",
        `${current}/tsconfig.json`,
        `${current}/../DefinitelyTyped/types/semver/index.d.ts`,
        "-o",
        `${current}/../dist/semver/index.d.ts`
    ],
    stdin: "null",
    stdout: undefined,
    stderr: undefined
});
command.spawn();
if((await command.output()).code !== 0){
    console.error("Failed to generate dist/semver/index.d.ts");
    Deno.exit(1);
}

await Deno.copyFile(`${current}/../dist/semver/index.d.ts`, `${current}/../dist/types/semver.d.ts`);
await Deno.copyFile(`${current}/../dist/rt/index.d.mts`, `${current}/../dist/types/rt.d.ts`);
await Deno.copyFile(`${current}/../dist/core/index.d.mts`, `${current}/../dist/types/core.d.ts`);

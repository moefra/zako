import { build } from 'tsdown'

let current = import.meta.dirname;

if(current == undefined){
    current = Deno.cwd();
}

const dist = `${current}/../../dist/`;

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

function makeDTs(input:string,output:string,module_name:string):void{
    const decoder = new TextDecoder("utf-8");
    let text = decoder.decode(await Deno.readFile(file));
    let resultText = `declare module "${module_name}" {\n\n${text}\n\n} // end module ${module_name}`;
    await Deno.writeTextFileSync(output,resultText);
}

await Deno.mkdir(`${dist}/types`, { recursive: true });
makeDTs(`${dist}/semver/index.d.ts`,`${dist}/types/semver.d.ts`, "zako:semver");
makeDTs(`${dist}/rt/index.d.ts`,`${dist}/types/rt.d.ts`, "zako:rt");
makeDTs(`${dist}/core/index.d.ts`,`${dist}/types/core.d.ts`, "zako:core");

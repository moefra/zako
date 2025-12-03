import { build } from 'tsdown'
import { copy } from "@std/fs/copy";

let current = import.meta.dirname;

if(current == undefined){
    current = Deno.cwd();
}

const dist = await Deno.realPath(`${current}/../../dist/`);

await Deno.remove(dist, { recursive: true }).catch(() => { /* ignore error */ });
await Deno.mkdir(dist, { recursive: true });

await copy(`${current}/../types/`, `${dist}/types/`);

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
        `${dist}/semver/index.d.mts`
    ],
    stdin: "null",
    stdout: undefined,
    stderr: undefined
});
command.spawn();
if((await command.output()).code !== 0){
    console.error("Failed to generate dist/semver/index.d.mts");
    Deno.exit(1);
}

async function makeDTs(input:string,output:string,module_name:string):Promise<void>{
    const decoder = new TextDecoder("utf-8");
    let input_data = await Deno.readFile(input);
    let text = decoder.decode(input_data);
    let resultText = "";
    if(module_name == "global"){
        resultText = text;
    }
    else {
        text = text.replaceAll(" declare ", " ").replaceAll("declare ", "");
        resultText = `declare module "${module_name}" {\n\n${text}\n\n} // end module ${module_name}`;
    }
    await Deno.writeTextFileSync(output,resultText);
}

await makeDTs(`${dist}/semver/index.d.mts`,`${dist}/types/semver.d.ts`, "zako:semver");
await makeDTs(`${dist}/rt/rt.d.mts`,`${dist}/types/rt.d.ts`, "zako:rt");
await makeDTs(`${dist}/core/core.d.mts`,`${dist}/types/core.d.ts`, "zako:core");
await makeDTs(`${dist}/global/global.d.mts`,`${dist}/types/global.d.ts`, "global");

let modDTs = "/// <reference no-default-lib=\"true\" />\n"
modDTs += "/// <reference lib=\"esnext\" />\n";
for await (const file of Deno.readDir(`${dist}/types/`)) {
    modDTs += `/// <reference path="./${file.name}" />\n`;
}
modDTs += "\nexport {};";
await Deno.writeTextFile(`${dist}/types/mod.ts`, modDTs);

await Deno.copyFile(`${current}/jsr.json`, `${dist}/types/jsr.json`);

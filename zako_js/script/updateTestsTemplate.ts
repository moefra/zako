// this script should be ran after build.ts
import * as fs from "node:fs/promises";

const current = import.meta.dir;
const dist = `${current}/../../dist`;
const testPath = `${current}/../../tests`;

// update template/tsconfig.json to tests/tsconfig.json and tests/tsconfig.*.json
async function removeOld(){
    const files = await fs.readdir(testPath);
    for(const file of files){
        if(file.startsWith("tsconfig.")){
            await fs.unlink(`${testPath}/${file}`);
        }
    }
}

async function updateTsconfig() {
    const templateTsconfig = await fs.readdir(`${dist}/template/`);
    for (const file of templateTsconfig) {
        if (file.startsWith("tsconfig")) {
            const content = await fs.readFile(
                `${dist}/template/${file}`,
                "utf-8",
            );
            await fs.writeFile(`${testPath}/${file}`, content, "utf-8");
        }
    }
}

// copy dist/types/* to tests/.zako/types/
async function updateTsTypeFile(){
    const targetDir = `${testPath}/.zako/types/zako/`;
    await fs.rm(targetDir, { recursive: true, force: true });
    await fs.mkdir(targetDir, { recursive: true });
    await fs.cp(`${dist}/types/`, targetDir, { recursive: true });
}

await removeOld();
await updateTsconfig();
await updateTsTypeFile();

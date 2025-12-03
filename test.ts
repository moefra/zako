import { dirname, fromFileUrl, join } from "jsr:@std/path";

/**
 * Helper function to run a subprocess.
 * Exits the script with code 1 if the command fails (replicating `if(-not $?)`).
 */
async function runCommand(cmd: string, args: string[]) {
    const command = new Deno.Command(cmd, {
        args: args,
        stdout: undefined,
        stderr: undefined,
        stdin: "null"
    });

    if (!(await command.spawn()).success) {
        console.error(`Error: Command failed -> ${cmd} ${args.join(" ")}`);
        Deno.exit(1);
    }
}

async function main() :Promise<void>{
    const currentDir = import.meta.dirname;
    const testsPath = join(currentDir, "tests");

    if(Deno.args.length != 1){
        console.error("Usage: zako deno run --allow-all test.ts <path-to-zako-executable>");
        Deno.exit(1);
    }

    console.log(`Starting test suite in: ${testsPath}`);

    try {
        const testEntries = [];
        for await (const entry of Deno.readDir(testsPath)) {
            testEntries.push(entry);
        }

        testEntries.sort((a, b) => a.name.localeCompare(b.name));

        for (const test of testEntries) {
            console.log(`Running test: ${test.name}`);

            const contextPath = join(testsPath, test.name);

            const argFilePath = `@${join(contextPath, "argfile")}`;

            await runCommand(Deno.args[0], [
                "-C",
                contextPath,
                argFilePath,
            ]);
        }

        console.log("All tests completed successfully.");
        Deno.exit(0);

    } catch (error) {
        throw error;
    }
}

if (import.meta.main) {
    await main();
}

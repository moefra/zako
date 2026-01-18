#! /usr/bin/env bun

import { options } from "./build.json";
const current = import.meta.dir;

let cliOpt = [];

if(Bun.argv.length >= 3){
    cliOpt = Bun.argv.slice(2);
}

const proc = Bun.spawn([
    "cargo","+nightly", "run", "-p", "zako-cli","-Z","unstable-options",
    ...options,
    "--",
    ...cliOpt
], {
    cwd: current,
});

await proc.exited;


#! /usr/bin/env bun

import { options } from "./build.json";
const current = import.meta.dir;

let opts = [];

if(Bun.argv.length >= 3){
    opts = Bun.argv.slice(2);
}

const proc = Bun.spawn([
    "cargo","+nightly", "build", "-p", "zako-cli","-Z","unstable-options",
    ...options,
    ...opts
], {
    cwd: current,
});

await proc.exited;

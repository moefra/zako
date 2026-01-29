#! /usr/bin/env bun

import { options } from "./build.json";

const current = import.meta.dir;

let opts = [];

if(Bun.argv.length >= 3){
    opts = Bun.argv.slice(2);
}

const proc = Bun.spawn([
    "cargo", "+nightly", "build", "-p", "zako-core",
    ...options,
    ...opts
], {
    cwd: current,
});

await proc.exited;

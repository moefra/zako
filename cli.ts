#! /usr/bin/env bun

import { options } from "./build.json";
const current = import.meta.dir;

const proc = Bun.spawn([
    "cargo","+nightly", "build", "-p", "zako-cli","-Z","unstable-options",
    ...options,
], {
    cwd: current,
});

await proc.exited;

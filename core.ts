#! /usr/bin/env bun

import { options } from "./build.json";

const current = import.meta.dir;

const proc = Bun.spawn([
    "cargo", "+nightly", "build", "-p", "zako-core",
    ...options,
], {
    cwd: current,
});

await proc.exited;

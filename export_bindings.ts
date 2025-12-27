#! /usr/bin/env bun

import { rm } from "node:fs/promises";

const current = import.meta.dir;

// remove old bindings
await rm(`${current}/zako_core/bindings`, { recursive: true, force: true });

const proc = Bun.spawn([
    "cargo", "+nightly", "test", "--package", "zako-core", "--tests", "export_bindings"
], {
    cwd: current,
});

await proc.exited;


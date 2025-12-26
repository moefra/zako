#! /usr/bin/env bun

const current = import.meta.dir;
 
const proc = Bun.spawn([
    "cargo", "+nightly", "test", "--package", "zako-core", "--tests", "export_bindings"
], {
    cwd: current,
});
 
await proc.exited; 


#! /usr/bin/env bun

const current = import.meta.dir;
 
const proc = Bun.spawn([
    "cargo", "+nightly", "test", "--workspace"
], {
    cwd: current,
});
 
await proc.exited; 


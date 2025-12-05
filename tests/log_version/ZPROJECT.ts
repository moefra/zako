
import * as syscall from "zako:syscall";

syscall.log("trace","hello world");

import * as core from "zako:core";

core.trace(core.version);
core.debug(core.version);
core.info(core.version);
core.warn(core.version);
core.error(core.version);

console.trace(core.version);
console.debug(core.version);
console.log(core.version);
console.info(core.version);
console.warn(core.version);
console.error(core.version);
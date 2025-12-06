// @ts-nocheck

import * as _rt from "zako:rt";

export function log(
        level: "trace" | "debug" | "info" | "warn" | "error",
        message: string,
    ): void{
    Deno.core.ops.syscall_log(level, message);
}

export const version: string = Deno.core.ops.syscall_version() as string;

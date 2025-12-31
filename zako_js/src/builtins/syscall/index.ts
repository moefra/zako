// @ts-nocheck

// to instantiate rt module
import * as _rt from "zako:rt";

const syscalls = Deno.core.ops as any;

export default syscalls;

export function log(
        level: "trace" | "debug" | "info" | "warn" | "error",
        message: string,
    ): void{
    syscalls.syscall_core_log(level, message);
}

export const version: string = syscalls.syscall_core_version() as string;

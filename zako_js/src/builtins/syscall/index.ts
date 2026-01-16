// to instantiate rt module
import * as _rt from "zako:rt";

/**
 * @internal
 */
export interface Syscall{
    syscall_core_version():string
    syscall_core_log(level:string, msg:string):string
}

/**
 * @internal
 */
export const syscalls = (globalThis as any).Deno.core.ops as any as Syscall;

export function log(
        level: "trace" | "debug" | "info" | "warn" | "error",
        message: string,
    ): void{
    syscalls.syscall_core_log(level, message);
}

export const version: string = syscalls.syscall_core_version() as string;

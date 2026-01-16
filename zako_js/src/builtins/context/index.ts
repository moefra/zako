
import * as core_syscalls from "zako:syscall";

/**
 * @internal
 */
export interface ContextSyscall extends core_syscalls.Syscall{
    syscall_context_name():string
}
/**
 * @internal
 */
export const syscalls = core_syscalls as any as ContextSyscall;

export const name: "package" = syscalls.syscall_context_name() as any;

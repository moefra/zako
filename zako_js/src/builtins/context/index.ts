
import syscalls from "zako:syscall";

export const name: "package" = syscalls.syscall_context_name() as string as any;

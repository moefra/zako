declare module "zmake:syscall" {
    export function log(
        level: "trace" | "debug" | "info" | "warn" | "error",
        message: string,
    ): void;
    export const version: string;
}

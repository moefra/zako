declare module "zako:syscall" {
    export function log(
        level: "trace" | "debug" | "info" | "warn" | "error",
        message: string,
    ): void;
    export const version: string;
}

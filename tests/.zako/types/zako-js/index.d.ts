export {};
import type { Console } from "./console";
declare module "zako:console" {
    export * from "./console";
}
import type {} from "./semver";
declare module "zako:semver" {
    export * from "./semver";
}
import type {} from "./rt";
declare module "zako:rt" {
    export * from "./rt";
}
import type {} from "./syscall";
declare module "zako:syscall" {
    export * from "./syscall";
}
import type {} from "./core";
declare module "zako:core" {
    export * from "./core";
}
import type {} from "./global";
declare module "zako:global" {
    export * from "./global";
}
import type {} from "./system";
declare module "zako:system" {
    export * from "./system";
}
declare global { export const console: Console; }

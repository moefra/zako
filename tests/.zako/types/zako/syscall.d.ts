/// <reference path="./global.d.ts" />
declare const syscalls: any;
export default syscalls;
export declare function log(level: "trace" | "debug" | "info" | "warn" | "error", message: string): void;
export declare const version: string;

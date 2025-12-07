

import {
    trace as coreTrace,
    debug as coreDebug,
    info as coreInfo,
    warn as coreWarn,
    error as coreError
} from "zako:core";

import { type Console } from "zako:console";

function safeFormat(args: any[]): string {
    return args.map(arg => {
        try {
            if (typeof arg === "string") return arg;
            if (arg instanceof Error) return arg.stack || `${arg.name}: ${arg.message}`;
            if (typeof arg === "object" && arg !== null) {
                return JSON.stringify(arg, (_key, value) => {
                    if (typeof value === 'bigint') return value.toString() + 'n';
                    return value;
                }, 2);
            }
            return String(arg);
        } catch (e) {
            return `[Circular or Unserializable Object]`;
        }
    }).join(" ");
}

const zConsole:Console = {
    trace: (...args: any[]) => coreTrace(safeFormat(args)),
    debug: (...args: any[]) => coreDebug(safeFormat(args)),
    log:   (...args: any[]) => coreInfo(safeFormat(args)),
    info:  (...args: any[]) => coreInfo(safeFormat(args)),
    warn:  (...args: any[]) => coreWarn(safeFormat(args)),
    error: (...args: any[]) => coreError(safeFormat(args)),
};

Object.defineProperty(globalThis, "console", {
    value: zConsole,
    writable: true,
    enumerable: false,
    configurable: true,
});

declare global {
    const console: Console;
}

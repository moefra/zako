
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

    interface Math {
        /** @deprecated Zako: Math.random() is banned for determinism. */
        random(): never;
    }
    interface PromiseConstructor {
        /** @deprecated Zako: Promise.race() is banned to ensure unsealing protection. */
        race(...args: any[]): never;
    }
    interface SymbolConstructor {
        /** @deprecated Zako: Symbol constructor is banned. */
        new(description?: string | number): never;
        new():never;
    }
    interface String {
        /** @deprecated Zako: Do not use locale-sensitive APIs. */
        localeCompare(that: string): never;
        /** @deprecated Zako: Do not use locale-sensitive APIs. */
        toLocaleLowerCase(): never;
        /** @deprecated Zako: Do not use locale-sensitive APIs. */
        toLocaleUpperCase(): never;
    }
}

// Those are banned in runtime.
declare var Date: never;
declare var Intl: never;
declare var performance: never;
declare var Crypto: never;
declare var FinalizationRegistry: never;
declare var WeakRef: never;
declare var SharedArrayBuffer: never;
declare var Atomics: never;
declare var setTimeout: never;
declare var setInterval: never;
declare var CryptoKey: never;
declare var CryptoKeyPair: never;
declare var setTimeout: never;
declare var setInterval: never;
// ----------------------------

Object.defineProperty(Math, "random", {
    value: undefined,
    writable: false,
    configurable: false,
});

Object.defineProperty(Promise, "race", {
    value: undefined,
    writable: false,
    configurable: false,
});

const RawSymbol = globalThis.Symbol;
const BannedSymbol = function (_description?: string | number) {
    throw new Error("ZakoError: Symbol() constructor is banned. Use Symbol.for().");
};
Object.setPrototypeOf(BannedSymbol, RawSymbol);
BannedSymbol.prototype = RawSymbol.prototype;
globalThis.Symbol = BannedSymbol as any;

Object.defineProperty(String.prototype, "localeCompare", {
    value: undefined,
    writable: false,
    configurable: false,
});

Object.defineProperty(String.prototype, "toLocaleLowerCase", {
    value: undefined,
    writable: false,
    configurable: false,
});

Object.defineProperty(String.prototype, "toLocaleUpperCase", {
    value: undefined,
    writable: false,
    configurable: false,
});

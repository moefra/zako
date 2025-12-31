/// <reference path="./global.d.ts" />
import { type Console } from "zako:console";
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
        new (description?: string | number): never;
        new (): never;
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

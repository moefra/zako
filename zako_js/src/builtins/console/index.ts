/// <reference no-default-lib="true" />
/// <reference lib="esnext" />

export interface Console {
    trace(...args: any[]): void;
    debug(...args: any[]): void;
    log(...args: any[]): void;
    info(...args: any[]): void;
    warn(...args: any[]): void;
    error(...args: any[]): void;
}

declare module "zako:console" {
    export interface Console {
        trace(...args: any[]): void;
        debug(...args: any[]): void;
        log(...args: any[]): void;
        info(...args: any[]): void;
        warn(...args: any[]): void;
        error(...args: any[]): void;
    }
}

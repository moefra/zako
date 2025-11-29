declare module "node:console" {
    export interface Console {
        trace(...args: any): void;
        debug(...args: any): void;
        log(...args: any): void;
        warn(...args: any): void;
        error(...args: any): void;
    }
}

declare module "zmake:console" {
    export interface Console {
        trace(message: string): void;
        debug(message: string): void;
        log(message: string): void;
        warn(message: string): void;
        error(message: string): void;
    }
}

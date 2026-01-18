/// <reference path="./global.d.ts" />
export interface ConfigRegistry {
}
export declare function get_config<K extends keyof ConfigRegistry>(key: K): ConfigRegistry[K];

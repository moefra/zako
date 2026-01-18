
export interface ConfigRegistry {}

export function get_config<K extends keyof ConfigRegistry>(key: K): ConfigRegistry[K] {
    return (globalThis as any).CONFIG[key];
}


export interface ConfigRegistry {}

export function get_config<K extends keyof ConfigRegistry>(key: K): ConfigRegistry[K] {
    return (global as any).CONFIG[key];
}

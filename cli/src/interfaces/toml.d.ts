declare module 'toml' {
    export function parse<T extends object>(source: string): Partial<T> | undefined | null;
}

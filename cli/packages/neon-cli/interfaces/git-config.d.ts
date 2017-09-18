declare module 'git-config' {
    function internal(callback: internal.GitConfigCallback): void;
    namespace internal {
        export type Dict = { [key: string]: any };
        export type MaybeError = Error | null;
        export type GitConfigCallback = (err: MaybeError, config?: Dict) => void;
    }

    export = internal;
}

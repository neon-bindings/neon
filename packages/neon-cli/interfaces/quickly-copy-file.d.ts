declare module 'quickly-copy-file' {
    function internal(from: string, to: string): Promise<void>;
    export = internal;
}

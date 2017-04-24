declare module 'validate-npm-package-name' {
    namespace internal {
        export type Validation = {
            validForOldPackages: boolean,
            validForNewPackages: boolean,
            warnings?: string[],
            errors?: string[]
        }
    }
    function internal(name: string): internal.Validation;
    export = internal;
}

declare module 'validate-npm-package-license' {
    namespace internal {
        export type Validation = {
            validForOldPackages: boolean,
            validForNewPackages: boolean,
            warnings?: string[],
            spdx?: boolean,
            inFile?: string,
            unlicensed?: boolean
        }
    }
    function internal(license: string): internal.Validation;
    export = internal;
}

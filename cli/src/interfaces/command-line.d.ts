declare module 'command-line-usage' {

    namespace internal {
        export type Sections = Section | Section[];

        export type Section = ContentSection | OptionListSection;

        export type ContentSection = {
            header: string,
            content: string | string[] | Record<string, any>[],
            raw?: boolean
        };

        export type OptionListSection = {
            header: string,
            optionList: OptionDefinition[],
            group?: string | string[],
            hide?: string | string[]
        };

        export type OptionDefinition = {
            name: string,
            type: (value: any) => any,
            alias?: string,
            multiple?: boolean,
            defaultOption?: boolean,
            defaultValue?: any,
            group?: string | string[],
            description?: string,
            typeLabel?: string
        };
    }

    function internal(sections: internal.Sections): string;

    export = internal;
}

declare module 'command-line-commands' {
    function internal(commands: string | null | (string | null)[], argv?: string[]): { command: string, argv: string[] };
    export = internal;
}

declare module 'command-line-args' {

    namespace internal {
        export type CommandLineArgsOptions = {
            argv?: string[],
            partial?: boolean
        };

        export type OptionDefinition = {
            name: string,
            type: (value: any) => any,
            alias?: string,
            multiple?: boolean,
            defaultOption?: boolean,
            defaultValue?: any,
            group?: string | string[],
            description?: string,
            typeLabel?: string
        };
    }

    function internal(optionDefinitions: internal.OptionDefinition[],
                      options?: internal.CommandLineArgsOptions)
        : Record<string, unknown>;

    export = internal;
}

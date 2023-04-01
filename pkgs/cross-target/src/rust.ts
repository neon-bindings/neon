import * as node from './node';

export const VENDORS: string[] = [
    'unknown', 'apple', 'pc'
];

export const OSES: string[] = [
    'darwin', 'linux', 'freebsd', 'openbsd', 'windows'
];

export const ABIS: string[] = [
    'gnu', 'msvc'
];

export class Platform {
    private _vendor: string;
    private _sys: string;
    private _abi: string | undefined;

    constructor(vendor: string, sys: string, abi?: string) {
        if (!VENDORS.includes(vendor)) {
            throw new RangeError(`unsupported Rust platform vendor: ${vendor}`);
        }
        if (!OSES.includes(sys)) {
            throw new RangeError(`unsupported Rust OS: ${sys}`);
        }
        if (abi && !ABIS.includes(abi)) {
            throw new RangeError(`unsupported Rust ABI: ${abi}`);
        }
        this._vendor = vendor;
        this._sys = sys;
        this._abi = abi;
    }

    vendor(): string { return this._vendor; }
    sys(): string { return this._sys; }
    abi(): string | undefined  { return this._abi; }

    toString(): string {
        const base = `${this._vendor}-${this._sys}`;
        return this._abi
            ? base + `-${this._abi}`
            : base;
    }

    equals(other: Platform): boolean {
        return (this._vendor === other._vendor)
            && (this._sys === other._sys)
            && (this._abi === other._abi);
    }
}

export type Arch = string;

export class Target {
    private _arch: Arch;
    private _platform: Platform;

    constructor(arch: Arch, platform: Platform) {
        if (!ARCHES.includes(arch)) {
            throw new RangeError(`unsupported Rust architecture: ${arch}`);
        }
        this._arch = arch;
        this._platform = platform;
    }

    static fromNode(target: node.Target): Target {
        const arch: node.Arch = target.arch();
        const platform: node.Platform = target.platform();

        const rustArch = ARCH_MAP[arch];
        if (!rustArch) {
            throw new RangeError(`unsupported Node architecture: ${arch}`);
        }

        const rustPlatform = PLATFORM_MAP[platform];
        if (!rustPlatform) {
            throw new RangeError(`unsupported Node platform: ${platform}`);
        }

        return new Target(rustArch, rustPlatform);
    }

    static current(): Target {
        return Target.fromNode(node.Target.current());
    }

    static parse(source: string): Target {
        const parts = source.split(/-/);

        if (parts.length === 3) {
            const [arch, vendor, sys] = parts;
            return new Target(arch, new Platform(vendor, sys));
        }

        if (parts.length === 4) {
            const [arch, vendor, sys, abi] = parts;
            return new Target(arch, new Platform(vendor, sys, abi));
        }

        throw new SyntaxError(`invalid Rust target: ${source}`);
    }

    toString(): string {
        return `${this._arch}-${this._platform.toString()}`;
    }

    equals(other: Target): boolean {
        return (this._arch === other._arch)
            && this._platform.equals(other._platform);
    }

    templateMetadata(): unknown {
        // FIXME: other metadata e.g. node arch/platform
        return {
            target: this.toString(),
            arch: this._arch,
            vendor: this._platform.vendor(),
            sys: this._platform.sys(),
            abi: this._platform.abi() ?? null
        }
    }
}

// https://doc.rust-lang.org/nightly/rustc/platform-support.html
// https://rust-lang.github.io/rustup-components-history/
// https://clang.llvm.org/docs/CrossCompilation.html#target-triple
export const ARCH_MAP: Record<node.Arch, Arch> = {
    'arm64': 'aarch64',
    'ia32':  'i686',
    'ppc':   'powerpc',
    'ppc64': 'powerpc64',
    's390x': 's390x',
    'x64':   'x86_64'
};

export const PLATFORM_MAP: Record<node.Platform, Platform> = {
    'darwin':  new Platform('apple',   'darwin'         ),
    'freebsd': new Platform('unknown', 'freebsd'        ),
    'linux':   new Platform('unknown', 'linux',   'gnu' ),
    'openbsd': new Platform('unknown', 'openbsd'        ),
    'win32':   new Platform('pc',      'windows', 'msvc')
};

// https://doc.rust-lang.org/nightly/rustc/platform-support.html
// https://rust-lang.github.io/rustup-components-history/
// https://clang.llvm.org/docs/CrossCompilation.html#target-triple
export const ARCHES: Arch[] = [
    'aarch64',
    'i386', 'i586', 'i686', 'x86_64',
    'arm', 'armv5te', 'armv7', 'armv7s',
    'armebv7r', 'armv7a', 'armv7r',
    'asmjs',
    'wasm32',
    'mips', 'mips64', 'mipsel',
    'mips64el', 'mipsisa32r6', 'mipsisa32r6el', 'mipsisa64r6', 'mipsisa64r6el',
    'powerpc', 'powerpc64', 'powerpc64le',
    'sparc', 'sparc64', 'sparcv9',
    'thumbv6m', 'thumbv7em', 'thumbv7m',
    'thumbv7neon', 'thumbv8m.base', 'thumbv8m.main',
    's390x', 'le32', 'msp430',
    'nvptx64'
];

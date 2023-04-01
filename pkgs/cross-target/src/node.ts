export type Arch = string;

// https://nodejs.org/api/process.html#processarch
const ARCHES: Arch[] = [
    'arm', 'arm64', 'ia32', 'mips', 'mipsel', 'ppc', 'ppc64', 's390', 's390x', 'x64'
];

export type Platform = string;

// https://nodejs.org/api/process.html#processplatform
const PLATFORMS: Platform[] = [
    'aix', 'darwin', 'freebsd', 'linux', 'openbsd', 'sunos', 'win32'
];

export class Target {
    private _arch: Arch;
    private _platform: Platform;
    
    constructor(arch: Arch, platform: Platform) {
        if (!ARCHES.includes(arch)) {
            throw new RangeError(`unsupported Node arch: ${arch}`);
        }
        if (!PLATFORMS.includes(platform)) {
            throw new RangeError(`unsupported Node platform: ${platform}`);
        }
        this._arch = arch;
        this._platform = platform;
    }

    static current(): Target { return new Target(process.arch, process.platform); }

    arch(): Arch { return this._arch; }
    platform(): Platform { return this._platform; }
}

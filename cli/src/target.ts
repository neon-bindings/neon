import * as rust from './rust';
import path from 'path';
import Crate from './crate';
import BuildSettings from './build-settings';
import { rimraf } from './async/rimraf';

const LIB_PREFIX: Record<string, string> = {
  'darwin':  "lib",
  'freebsd': "lib",
  'linux':   "lib",
  'sunos':   "lib",
  'win32':   ""
};

const LIB_SUFFIX: Record<string, string> = {
  'darwin':  ".dylib",
  'freebsd': ".so",
  'linux':   ".so",
  'sunos':   ".so",
  'win32':   ".dll"
};

export type TargetOptions = {
  release?: boolean,
  arch?: string
};

/** The Rust build artifacts for a single build target of a Neon crate. */
export default class Target {
  readonly crate: Crate;
  readonly release: boolean;
  readonly arch: string;
  readonly triple: string;
  readonly subdirectory: string;
  readonly root: string;
  readonly dylib: string;

  constructor(crate: Crate, options: TargetOptions = {}) {
    let { release = true, arch = process.env.npm_config_arch || process.arch } = options;
    this.crate = crate;
    this.release = release;
    this.arch = arch;

    if (process.platform === 'win32') {
      this.triple = (arch === 'ia32') ? 'i686-pc-windows-msvc' : 'x86_64-pc-windows-msvc';
    } else {
      this.triple = '';
    }

    if (process.env.CARGO_BUILD_TARGET) {
      this.triple = process.env.CARGO_BUILD_TARGET;
    }

    this.subdirectory = path.join(this.triple, release ? 'release' : 'debug');
    this.root = path.resolve(crate.project.targetDirectory, this.subdirectory);

    let prefix = LIB_PREFIX[process.platform];
    let suffix = LIB_SUFFIX[process.platform];
    this.dylib = path.resolve(this.root, prefix + crate.name + suffix);
  }

  async clean() {
    // Remove the directory associated with this target.
    const absolutePathSubdir = path.resolve(this.crate.root, 'target', this.subdirectory);
    await rimraf(absolutePathSubdir);

    // If this target was the active target, remove the addon.
    if (this.crate.artifacts.haveActivated(this.subdirectory)) {
      await this.crate.removeAddon();
    }

    // Update the build state.
    this.crate.artifacts.delete(this.subdirectory);
    this.crate.saveArtifacts();
  }

  async build(toolchain: rust.Toolchain,
              settings: BuildSettings,
              additionalArgs: string[])
  {
    let releaseFlags = this.release ? ["--release"] : [];
    let targetFlags = this.triple ? ["--target=" + this.triple] : [];

    let args = ['build'].concat(releaseFlags, targetFlags, additionalArgs);

    try {
      let result = await rust.spawn("cargo", args, toolchain, {
        cwd: this.crate.root,
        stdio: 'inherit'
      });

      if (result !== 0) {
        throw new Error("cargo build failed");
      }

      this.crate.artifacts.activate(this.subdirectory, settings);

      return result;
    } finally {
      this.crate.saveArtifacts();
    }
  }

  inState(settings: BuildSettings) {
    let savedSettings = this.crate.artifacts.lookup(this.subdirectory);
    return savedSettings && savedSettings.match(settings);
  }

};

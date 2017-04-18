import { remove } from './async/fs';
import * as rust from './rust';
import path from 'path';

const LIB_PREFIX = {
  'darwin':  "lib",
  'freebsd': "lib",
  'linux':   "lib",
  'sunos':   "lib",
  'win32':   ""
};

const LIB_SUFFIX = {
  'darwin':  ".dylib",
  'freebsd': ".so",
  'linux':   ".so",
  'sunos':   ".so",
  'win32':   ".dll"
};

// Represents the Rust build artifacts for a single build target of a Neon crate.
export default class Target {

  constructor(crate, options = {}) {
    let { release = true, arch = process.env.npm_config_arch || process.arch } = options;
    this.crate = crate;
    this.release = release;
    this.arch = arch;

    if (process.platform === 'win32') {
      this.triple = (arch === 'ia32') ? 'i686-pc-windows-msvc' : 'x86_64-pc-windows-msvc';
    } else {
      this.triple = '';
    }

    this.subdirectory = path.join(this.triple, release ? 'release' : 'debug');
    this.root = path.resolve(crate.root, 'target', this.subdirectory);

    let prefix = LIB_PREFIX[process.platform];
    let suffix = LIB_SUFFIX[process.platform];
    this.dylib = path.resolve(this.root, prefix + crate.name + suffix);
  }

  async clean() {
    // Remove the directory associated with this target.
    await remove(path.resolve(this.crate.root, 'target', this.subdirectory));

    // If this target was the active target, remove the addon.
    if (this.crate.artifacts.active === this.subdirectory) {
      await this.crate.removeAddon();
    }

    // Update the build state.
    this.crate.artifacts.delete(this.subdirectory);
    this.crate.saveArtifacts();
  }

  async build(toolchain, settings, abi = process.versions.modules) {
    let macos = process.platform === 'darwin';

    let command = macos ? 'rustc' : 'build';
    let releaseFlags = this.release ? ["--release"] : [];
    let extraFlags = macos ? ["--", "-C", "link-args=-Wl,-undefined,dynamic_lookup"] : [];
    let targetFlags = this.triple ? ["--target=" + this.triple] : [];

    let args = [command].concat(releaseFlags, extraFlags, targetFlags);

    try {
      let result = await rust.spawn("cargo", args, toolchain, {
        cwd: this.crate.root,
        stdio: 'inherit',
        // Pass the Node modules ABI version to the build as an environment variable.
        env: Object.assign({}, process.env, { NEON_NODE_ABI: abi })
      });

      if (result !== 0) {
        throw new Error("cargo build failed");
      }

      this.crate.artifacts.activate(this.subdirectory, settings);

      return result;
    } catch (e) {
      this.crate.artifacts.delete(this.subdirectory);

      throw e;
    } finally {
      this.crate.saveArtifacts();
    }
  }

  inState(settings) {
    let savedSettings = this.crate.artifacts.lookup(this.subdirectory);
    return savedSettings && savedSettings.match(settings);
  }

};

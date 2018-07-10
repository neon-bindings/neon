import * as path from 'path';
import Crate from './crate';
import Target from './target';
import BuildSettings from './build-settings';
import log from './log';
import { spawn } from './async/child_process';
import * as rust from './rust';

export type ProjectOptions = {
  crate?: string
};

/** A Neon project and its directory tree. */
export default class Project {
  readonly root: string;
  readonly crate: Crate;

  constructor(root: string, options: ProjectOptions = {}) {
    let { crate = 'native' } = options;
    this.root = root;
    this.crate = new Crate(this, { subdirectory: crate });
  }

  async build(toolchain: rust.Toolchain | null,
              release: boolean)
  {
    let target = new Target(this.crate, { release: release });
    let settings = BuildSettings.current(toolchain || 'default');

    // 1. Force a rebuild if build settings have changed.
    if (!target.inState(settings)) {
      log("forcing rebuild for new build settings");
      await target.clean();
    }

    // 2. Build the dylib.
    log("running cargo");
    await target.build(toolchain, settings);

    // 3. Copy the dylib as the main addon file.
    log("generating " + path.join(this.crate.subdirectory, this.crate.nodefile));
    await this.crate.finish(target.dylib);
  }

  async clean() {
    // 1. Do a `cargo clean` to delete the `target` directory.
    log("cargo clean");
    await spawn("cargo", ["clean"], { cwd: this.crate.root, stdio: 'inherit' });

    // 2. Remove the main addon file.
    log("remove " + path.join(this.crate.subdirectory, this.crate.nodefile));
    await this.crate.removeAddon();

    // 3. Clear the artifacts file.
    this.crate.resetArtifacts();
    this.crate.saveArtifacts();
  }

};

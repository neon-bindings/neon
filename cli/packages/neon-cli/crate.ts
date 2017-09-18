import * as TOML from 'toml';
import * as path from 'path';
import { readFileSync } from 'fs';
import { remove, copy } from './async/fs';
import Artifacts from './artifacts';
import Project from './project';

export type CrateOptions = {
  subdirectory?: string,
  nodefile?: string
};

// Represents the native crate inside a Neon project.
export default class Crate {
  /** The Neon project containing this crate. */
  readonly project: Project;
  /** The subpath of this crate relative to the Neon project root. */  
  readonly subdirectory: string;
  /** The subpath of the `.node` addon relative to this crate root. */
  readonly nodefile: string;
  /** The absolute path of this crate. */
  readonly root: string;
  /** The absolute path of the `.node` addon. */
  readonly addon: string;
  /** The crate name extracted from the manifest. */
  readonly name: string;
  /** The absolute path of the artifacts file. */
  readonly artifactsfile: string;
  /** The state of current build artifacts for each target. */
  readonly artifacts: Artifacts;

  constructor(project: Project, options: CrateOptions = {}) {
    let { subdirectory = 'native', nodefile = 'index.node' } = options;
    this.project = project;
    this.subdirectory = subdirectory;
    this.nodefile = nodefile;
    this.root = path.resolve(project.root, subdirectory);
    this.addon = path.resolve(this.root, nodefile);
    this.name = loadLibName(path.resolve(this.root, 'Cargo.toml'))
    this.artifactsfile =
      path.resolve(this.root, 'artifacts.json');
    this.artifacts = Artifacts.load(this.artifactsfile);
  }

  async finish(dylib: string) {
    await remove(this.addon);
    await copy(dylib, this.addon);
  }

  async removeAddon() {
    await remove(this.addon);
  }

  resetArtifacts() {
    this.artifacts.reset();
  }

  saveArtifacts() {
    this.artifacts.save(this.artifactsfile);
  }

}

function loadLibName(file: string): string {
  let metadata = TOML.parse(readFileSync(file, 'utf8'));

  if (!metadata || typeof metadata !== 'object' || !metadata.lib.name) {
    throw new Error("Cargo.toml does not contain a [lib] section with a 'name' field");
  }

  return metadata.lib.name;
}

import TOML from 'toml';
import path from 'path';
import { readFileSync } from 'fs';
import { remove, copy } from './async/fs';
import Artifacts from './artifacts';
import BuildSettings from './build-settings';
import log from './log';

// Represents the native crate inside a Neon project.
export default class Crate {

  constructor(project, options = {}) {
    let { subdirectory = 'native', nodefile = 'index.node' } = options;
    this.project = project;                                 // the Neon project containing this crate
    this.subdirectory = subdirectory;                       // the subpath of this crate relative to the Neon project root
    this.nodefile = nodefile;                               // the subpath of the .node addon relative to this crate root
    this.root = path.resolve(project.root, subdirectory);   // the absolute path of this crate
    this.addon = path.resolve(this.root, nodefile);         // the absolute path of the .node addon
    this.manifest =                                         // the parsed Cargo.toml manifest
      loadManifest(path.resolve(this.root, 'Cargo.toml'));
    this.name = this.manifest.lib.name;                     // the crate name extracted from the manifest
    this.artifactsfile =                                    // the absolute path of the artifacts file
      path.resolve(this.root, 'artifacts.json');
    this.artifacts = Artifacts.load(this.artifactsfile);    // the state of the previous build for each target
  }

  async finish(dylib) {
    await remove(this.addon);
    await copy(dylib, this.addon);
  }

  async removeAddon() {
    await remove(this.addon);
  }

  resetArtifacts() {
    this.artifacts = new Artifacts();
  }

  saveArtifacts() {
    this.artifacts.save(this.artifactsfile);
  }

}

function loadManifest(file) {
  let metadata = TOML.parse(readFileSync(file, 'utf8'));

  if (!metadata.lib.name) {
    throw new Error("Cargo.toml does not contain a [lib] section with a 'name' field");
  }

  return metadata;
}

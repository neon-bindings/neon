import BuildSettings from './build-settings';
import { writeFileSync } from 'fs';

// The artifacts file keeps track of the state of all build artifacts and the
// build settings that were used to build them. This includes the results of
// cargo builds in the Rust `target` directory as well as the main .node addon
// file.
//
// The artifacts file has the following structure:
//
// {
//   "active": string | null,
//   "targets": targets
// }
//
// The `"active"` property indicates the relative path within the `target`
// directory to the current active build, i.e., the build that was most
// recently copied as the main .node addon.
//
// The `targets` type is a JSON record that tracks the state of any build
// artifacts in the `target` directory.
//
// On Windows, this type is:
//
// {
//   "i686-pc-windows-msvc\\debug"?: settings,
//   "i686-pc-windows-msvc\\release"?: settings,
//   "x86_64-pc-windows-msvc\\debug"?: settings,
//   "x86_64-pc-windows-msvc\\release"?: settings
// }
//
// On Linux and macOS, this type is:
//
// {
//   "debug"?: settings,
//   "release"?: settings
// }
//
// The `settings` type is a JSON serialization of the `BuildSettings` class.

function parse(json) {
  let active = json.active;
  let targets = {};
  for (let target of Object.keys(json.targets)) {
    targets[target] = BuildSettings.fromJSON(json.targets[target]);
  }
  return new Artifacts(active, targets);
}

function jsonify(targets) {
  let json = {};
  for (let target of Object.keys(targets)) {
    json[target] = targets[target].toJSON();
  }
  return json;
}

export default class Artifacts {

  constructor(active = null, targets = {}) {
    this.active = active;
    this._targets = targets;
  }

  static load(file) {
    try {
      return parse(require(file));
    } catch (e) {
      return new Artifacts();
    }
  }

  toJSON() {
    return {
      active: this.active,
      targets: jsonify(this._targets)
    };
  }

  save(file) {
    writeFileSync(file, JSON.stringify(this.toJSON()));
  }

  lookup(path) {
    return this._targets[path];
  }

  activate(path, settings) {
    this._targets[path] = settings;
    this.active = path;
  }

  delete(path) {
    delete this._targets[path];

    // If the path being deleted was the active one, there's no more active path.
    if (this.active === path) {
      this.active = null;
    }
  }

}

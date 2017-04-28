import BuildSettings from './build-settings';
import { writeFileSync } from 'fs';
import Dict from 'ts-dict';
import * as JSON from 'ts-typed-json';

/**
 * The current state of build artifacts, for all targets.
 */
export default class Artifacts {
  /**
   * The relative path within the `target` directory to the current active build,
   * i.e., the build that was most recently copied as the main `.node` addon.
   */
  private active: string | null;

  /**
   * A table tracking the state of any build artifacts in the `target`
   * directory.
   * 
   * On Windows, this table has the type:
   * 
   * ```
   * {
   *   "i686-pc-windows-msvc\\debug"?: BuildSettings,
   *   "i686-pc-windows-msvc\\release"?: BuildSettings,
   *   "x86_64-pc-windows-msvc\\debug"?: BuildSettings,
   *   "x86_64-pc-windows-msvc\\release"?: BuildSettings
   * }
   * ```
   * 
   * On Linux and macOS, this table has the type:
   * 
   * ```
   * {
   *   "debug"?: BuildSettings,
   *   "release"?: BuildSettings
   * }
   * ```
   */
  private targets: Dict<BuildSettings>;

  constructor(active: string | null = null,
              targets: Dict<BuildSettings> = {})
  {
    this.active = active;
    this.targets = targets;
  }

  static load(file: string) {
    try {
      return Artifacts.fromJSON(JSON.loadSync(file));
    } catch (e) {
      return new Artifacts();
    }
  }

  static fromJSON(json: JSON.Value): Artifacts {
    if (!JSON.isObject(json)) {
      throw new TypeError("expected object, found " + (json === null ? "null" : typeof json));
    }
    let active = json.active;
    if (typeof active !== 'string' && active !== null) {
      throw new TypeError("json.active is not a string or null");
    }
    let jsonTargets = json.targets;
    if (!JSON.isObject(jsonTargets)) {
      throw new TypeError("json.targets is not an object");
    }
    let targets: Dict<BuildSettings> = {};
    for (let key of Object.keys(jsonTargets)) {
      targets[key] = BuildSettings.fromJSON(jsonTargets[key]);
    }
    return new Artifacts(active, targets);
  }

  toJSON(): JSON.Object {
    let targets: JSON.Object = {};
    for (let target of Object.keys(this.targets)) {
      targets[target] = this.targets[target].toJSON();
    }

    return {
      active: this.active,
      targets: targets
    };
  }

  save(file: string) {
    writeFileSync(file, JSON.stringify(this.toJSON()));
  }

  lookup(path: string) {
    return this.targets[path];
  }

  activate(path: string, settings: BuildSettings) {
    this.targets[path] = settings;
    this.active = path;
  }

  haveActivated(path: string): boolean {
    return this.active === path;
  }

  delete(path: string) {
    delete this.targets[path];

    // If the path being deleted was the active one, there's no more active path.
    if (this.active === path) {
      this.active = null;
    }
  }

  reset() {
    this.active = null;
    this.targets = {};
  }
}

import * as rust from './rust';
import Dict from 'ts-dict';
import * as JSON from 'ts-typed-json';

function isStringDict(x: JSON.Object): x is Dict<string | null> {
  for (let key of Object.keys(x)) {
    if (x[key] !== null && typeof x[key] !== 'string') {
      return false;
    }
  }
  return true;
}

export default class BuildSettings {
  private rustc: string;
  private nodeVersion: string;
  private env: Dict<string | null>;

  constructor(rustc: string, nodeVersion: string, env: Dict<string | null>) {
    this.rustc = rustc;
    this.nodeVersion = nodeVersion
    this.env = env;
  }

  match(other: BuildSettings) {
    if (other.nodeVersion !== this.nodeVersion) return false;
    return Object.keys(this.env).every(key => {
      return (!this.env[key] && !other.env[key]) ||
             (this.env[key] === other.env[key]);
    });
  }

  static getNodeVersion(toolchain: rust.Toolchain = 'default'): string {
    const nodeVersionResult = rust.spawnSync("node", ["--version"], toolchain);
    return nodeVersionResult.stdout
      .toString()
      .trim();
  }

  static current(toolchain: rust.Toolchain = 'default') {
    const rustcVersionResult = rust.spawnSync("rustc", ["--version"], toolchain);
    const nodeVersion = BuildSettings.getNodeVersion(toolchain);

    if (rustcVersionResult.error) {
      if (rustcVersionResult.error.message.includes("ENOENT")) {
        throw new Error('Rust is not installed or rustc is not in your path.');
      }
      throw rustcVersionResult.error;
    }

    let rustc = rustcVersionResult.stdout
      .toString()
      .trim();

    return new BuildSettings(rustc, nodeVersion, {
      npm_config_target:            process.env.npm_config_target || null,
      npm_config_arch:              process.env.npm_config_arch || null,
      npm_config_target_arch:       process.env.npm_config_target_arch || null,
      npm_config_disturl:           process.env.npm_config_disturl || null,
      npm_config_runtime:           process.env.npm_config_runtime || null,
      npm_config_build_from_source: process.env.npm_config_build_from_source || null,
      npm_config_devdir:            process.env.npm_config_devdir || null
    });
  }

  static fromJSON(value: JSON.Value): BuildSettings {
    if (!JSON.isObject(value)) {
      throw new TypeError("value must be an object");
    }
    let { rustc, env, nodeVersion } = value;
    if (typeof rustc !== 'string') {
      throw new TypeError("value.rustc must be a string");
    }
    if (!('nodeVersion' in value)) {
      nodeVersion = BuildSettings.getNodeVersion(null);
    } else {
      if (typeof nodeVersion !== 'string') {
        throw new TypeError("value.nodeVersion must be a string");
      }
    }
    if (!JSON.isObject(env)) {
      throw new TypeError("value.env must be an object");
    }
    if (!isStringDict(env)) {
      throw new TypeError("value.env must be a string dict");
    }
    return new BuildSettings(rustc, nodeVersion, env);
  }

  toJSON(): JSON.Object {
    return {
      "rustc": this.rustc,
      "nodeVersion": this.nodeVersion,
      "env": this.env
    };
  }

}

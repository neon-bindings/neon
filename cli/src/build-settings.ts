import * as rust from './rust';
import * as JSON from 'ts-typed-json';

function isStringDict(x: JSON.Object): x is Record<string, string | null> {
  for (let key of Object.keys(x)) {
    if (x[key] !== null && typeof x[key] !== 'string') {
      return false;
    }
  }
  return true;
}

export default class BuildSettings {
  private rustc: string;
  private nodeVersion: string | null;
  private env: Record<string, string | null>;

  constructor(rustc: string, nodeVersion: string | null, env: Record<string, string | null>) {
    this.rustc = rustc;
    this.nodeVersion = nodeVersion;
    this.env = env;
  }

  match(other: BuildSettings) {
    if (other.nodeVersion !== this.nodeVersion) {
      return false;
    }
    return Object.keys(this.env).every(key => {
      return (!this.env[key] && !other.env[key]) ||
             (this.env[key] === other.env[key]);
    });
  }

  static getNodeVersion(): string {
    return process.version;
  }

  static current(toolchain: rust.Toolchain = 'default') {
    let rustcVersionResult = rust.spawnSync("rustc", ["--version"], toolchain);
    let nodeVersion = BuildSettings.getNodeVersion();

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
      npm_config_devdir:            process.env.npm_config_devdir || null,
      npm_config_node_engine:       process.env.npm_config_node_engine || null,
      npm_config_nodedir:           process.env.npm_config_nodedir || null,
      npm_config_node_gyp:          process.env.npm_config_node_gyp || null,
      npm_config_platform:          process.env.npm_config_platform || null
    });
  }

  static fromJSON(value: JSON.Value): BuildSettings {
    value = JSON.asObject(value, "value")
    let { rustc, env, nodeVersion } = value;
    if (typeof rustc !== 'string') {
      throw new TypeError("value.rustc must be a string");
    }
    if ('nodeVersion' in value) {
      if (typeof nodeVersion !== 'string' && nodeVersion !== null) {
        throw new TypeError("value.nodeVersion must be a string or null");
      }
    } else {
      nodeVersion = null;
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

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
  private env: Dict<string | null>;

  constructor(rustc: string, env: Dict<string | null>) {
    this.rustc = rustc;
    this.env = env;
  }

  match(other: BuildSettings) {
    return Object.keys(this.env).every(key => {
      return (!this.env[key] && !other.env[key]) ||
             (this.env[key] === other.env[key]);
    });
  }

  static current(toolchain: rust.Toolchain) {
    const spawnResult = rust.spawnSync("rustc", ["--version"], toolchain);

    if (spawnResult.error) {
      if (spawnResult.error.message.includes("ENOENT")) {
        throw new Error('Rust is not installed or rustc is not in your path.');
      }
      throw spawnResult.error;
    }

    let rustc = spawnResult.stdout
      .toString('utf8')
      .trim();

    return new BuildSettings(rustc, {
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
      throw new TypeError("value is not an object");
    }
    let rustc = value.rustc;
    let env = value.env;
    if (typeof rustc !== 'string') {
      throw new TypeError("value.rustc is not a string");
    }
    if (!JSON.isObject(env)) {
      throw new TypeError("value.env is not an object");
    }
    if (!isStringDict(env)) {
      throw new TypeError("value.env is not a string dict");
    }
    return new BuildSettings(rustc, env);
  }

  toJSON(): JSON.Object {
    return {
      "rustc": this.rustc,
      "env": this.env
    };
  }

}

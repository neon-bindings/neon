import * as rust from './rust';

export default class BuildSettings {

  constructor(rustc, env) {
    this.rustc = rustc;
    this.env = env;
  }

  match(other) {
    return Object.keys(this.env).every(key => {
      return (!this.env[key] && !other.env[key]) ||
             (this.env[key] === other.env[key]);
    });
  }

  static current(toolchain) {
    let rustc = rust.spawnSync("rustc", ["--version"], toolchain)
      .stdout
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

  static fromJSON(obj) {
    return new BuildSettings(obj.rustc, obj.env);
  }

  toJSON() {
    return {
      "rustc": this.rustc,
      "env": this.env
    };
  }

}

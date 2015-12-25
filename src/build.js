import fs from 'fs';
import child from 'child_process';
import path from 'path';
import handlebars from 'handlebars';

const NODE_GYP = path.resolve(path.resolve(path.resolve(path.resolve(path.dirname(require.resolve('node-gyp')), '..'), '..'), '.bin'), 'node-gyp');

const TEMPLATES_DIR = path.resolve(path.resolve(__dirname, ".."), "templates");

const TEMPLATE = handlebars.compile(fs.readFileSync(path.resolve(TEMPLATES_DIR, "binding.gyp.hbs"), 'utf8'), { noEscape: true });

class Build {
  constructor(project, rust, type) {
    this.project = project;
    this.rust = rust;
    this.type = type;
  }

  generateGypfile() {
    let release = this.type === 'release';

    let context = {
      project: {
        name: this.project.libName,
        rust: { inputs: this.project.getRustInputs() }
      },
      build: {
        cfg: {
          gyp: release ? "Release" : "Debug",
          cargo: this.type
        },
        release: release,
        cargo: { cmd: this.rust === 'default' ? [] : ["multirust", "run", this.rust] }
      }
    };

    fs.writeFileSync(this.project.getGypfilePath(), TEMPLATE(context));
  }

  isStale() {
    try {
      let gypfile = this.project.getGypfileContents();
      // FIXME: check if the set of .rs source files has changed?
      return (gypfile.target_defaults.default_configuration.toLowerCase() !== this.type) ||
             (gypfile.targets[0].target_name !== this.project.libName);
    } catch (e) {
      return true;
    }
    return false;
  }

  run() {
    // 1. Generate the gypfile
    this.generateGypfile();

    // 2. `node-gyp rebuild`
    let options = { cwd: this.project.root, stdio: 'inherit' };
    let result = child.spawnSync(NODE_GYP, ["rebuild"], options);
    if (result.status !== 0) {
      process.exit(result.status);
    }
  }
}

export default function(project, rust, type) {
  return new Build(project, rust, type);
};

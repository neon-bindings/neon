var fs = require('fs');
var child = require('child_process');
var path = require('path');
var handlebars = require('handlebars');

var NODE_GYP = path.resolve(path.resolve(path.resolve(path.resolve(path.dirname(require.resolve('node-gyp')), '..'), '..'), '.bin'), 'node-gyp');

var TEMPLATES_DIR = path.resolve(path.resolve(__dirname, ".."), "templates");

var TEMPLATE = handlebars.compile(fs.readFileSync(path.resolve(TEMPLATES_DIR, "binding.gyp.hbs"), 'utf8'), { noEscape: true });

function parseRoot(root, str) {
  return str.split("/").slice(0, -1).reduce(function(base, rel) { return path.resolve(base, rel); }, root);
}

function Build(project, rust, type) {
  this.project = project;
  this.rust = rust;
  this.type = type;
}

Build.prototype.generateGypfile = function generateGypfile() {
  var release = this.type === 'release';

  var context = {
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
};

Build.prototype.isStale = function isStale() {
  try {
    var gypfile = this.project.getGypfileContents();
    // FIXME: check if the set of .rs source files has changed?
    return (gypfile.target_defaults.default_configuration.toLowerCase() !== this.type) ||
           (gypfile.targets[0].target_name !== this.project.libName);
  } catch (e) {
    return true;
  }
  return false;
};

Build.prototype.run = function run() {
  // 1. Generate the gypfile
  this.generateGypfile();

  // 2. `node-gyp rebuild`
  var options = { cwd: this.project.root, stdio: 'inherit' };
  var result = child.spawnSync(NODE_GYP, ["rebuild"], options);
  if (result.status !== 0) {
    process.exit(result.status);
  }
};

module.exports = exports = function(project, rust, type) {
  return new Build(project, rust, type);
};

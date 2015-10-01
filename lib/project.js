var fs = require('fs');
var toml = require('toml');
var path = require('path');
var find = require('find');

function Project(root, manifest, libName) {
  this._manifestContents = null;
  this._gypfileContents = null;
  this.root = root;
  this.manifest = manifest || root + "/Cargo.toml";
  this.libName = libName || this.getManifestContents().package.name.replace('-', '_');
}

Project.prototype.getManifestContents = function getManifestContents() {
  return this._manifestContents || (this._manifestContents = toml.parse(fs.readFileSync(this.manifest, 'utf8')));
};

Project.prototype.getGypfileContents = function getGypfileContents() {
  return this._gypfileContents || (this._gypfileContents = JSON.parse(fs.readFileSync(this.getGypfilePath(), 'utf8').replace(/#.*/g, "")));
};

Project.prototype.getAddonPath = function getAddonPath() {
  return path.resolve(this.root, "build/" + this.getGypfileContents().target_defaults.default_configuration + "/" + this.libName + ".node");
};

Project.prototype.getGypfilePath = function getGypfilePath() {
  return path.resolve(this.root, "binding.gyp");
};

// Find all .rs source files in the project.
Project.prototype.getRustInputs = function getRustInputs() {
  var roots = [path.resolve(this.root, 'src')];
  var manifest = this.getManifestContents();
  if (manifest.bin && manifest.bin.path) {
    roots.push(parseRoot(this.root, manifest.bin.path));
  }
  if (manifest.lib && manifest.lib.path) {
    roots.push(parseRoot(this.root, manifest.lib.path));
  }
  var inputs = [];
  roots.forEach(function(root) {
    find.fileSync(/\.rs$/, root).forEach(function(file) {
      inputs.push(path.relative(this.root, file));
    }, this);
  }, this);
  return inputs;
};


module.exports = exports = function(root, manifest, libName) {
  return new Project(root, manifest, libName);
};

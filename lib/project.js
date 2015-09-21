var fs = require('fs');
var toml = require('toml');

function Project(root, manifest, libName) {
  this._manifestContents = null;
  this.root = root;
  this.manifest = manifest;
  this.libName = libName || this.getManifestContents().package.name.replace('-', '_');
}

Project.prototype.getManifestContents = function getManifestContents() {
  if (this._manifestContents) {
    return this._manifestContents;
  }
  this._manifestContents = toml.parse(fs.readFileSync(this.manifest, 'utf8'));
  return this._manifestContents;
};

module.exports = exports = function(root, manifest, libName) {
  return new Project(root, manifest, libName);
};

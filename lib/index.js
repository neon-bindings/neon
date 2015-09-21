var path = require('path');
var bindings = require('bindings');

function getRoot() {
  return bindings.getRoot(bindings.getFileName(__filename));
}

// mode:      'debug' | 'release' = 'release'
// root:      string = nearest containing directory of caller with package.json or node_modules
// manifest:  $root/Cargo.toml
// name:      TOML($manifest).package.name
// multirust: 'nightly' | 'stable' | undefined = 'nightly' (eventually will switch to undefined)

function Bindings(opts) {
  if (typeof opts === 'string') {
    opts = { name: opts };
  } else if (!opts) {
    opts = {};
  }
  var root = opts.root || getRoot();
  this.root = root;
  this.project = require('./project')(root, root + "/Cargo.toml", opts.name); // FIXME: opts.manifest
  this.mode = 'release'; // FIXME: opts.mode
  // FIXME: opts.multirust
}

Bindings.prototype.getAddonPath = function getAddonPath() {
  return path.resolve(this.root, "build/" + this.mode[0].toUpperCase() + this.mode.substring(1) + "/" + this.project.libName + ".node");
};

// function fileExists(filename) {
//   try {
//     return fs.statSync(filename).isFile();
//   } catch (e) {
//     return false;
//   }
// }

module.exports = exports = function rustBindings(opts) {
  var b = new Bindings(opts);

  var addonPath = b.getAddonPath();
  // if (!b.fileExists(addonPath)) {
  //   b.build();
  // }
  return require(addonPath);
};

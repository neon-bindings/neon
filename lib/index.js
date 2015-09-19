var fs = require('fs');
var bindings = require('bindings');
var toml = require('toml');
var beautify = require('js-beautify').js_beautify;
var child = require('child_process');
var path = require('path');

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
  this._manifestContents = null;
  this.mode = opts.mode || 'release';
  this.root = opts.root || bindings.getRoot(bindings.getFileName(__filename));
  this.manifest = opts.manifest || this.root + "/Cargo.toml";
  this.name = opts.name || this.getManifestContents().package.name;
  this.multirust = opts.multirust || 'nightly';
}

Bindings.prototype.getManifestContents = function getManifestContents() {
  if (this._manifestContents) {
    return this._manifestContents;
  }
  this._manifestContents = toml.parse(fs.readFileSync(this.manifest, 'utf8'));
  return this._manifestContents;
};

Bindings.prototype.getAddonSource = function getAddonSource() {
  return "/* THIS FILE WAS AUTOMATICALLY GENERATED. DO NOT EDIT. */\n\n"
       + "#include <nan.h>\n\n"
       + "extern \"C\" void node_main(v8::Local<v8::Object> module);\n\n"
       + "NODE_MODULE(" + this.name + ", node_main)\n";
};

Bindings.prototype.getGypSource = function getGypSource() {
  return beautify(JSON.stringify({
    "targets": [{
      "target_name": this.name,
      "sources": ["addon.cc"],
      "include_dirs": [this.getNanPath()], //["<!(node -e \"require('rust-bindings').nan()\")"],
      "libraries": ["../" + this.getDynamicLibPath()]
    }]
  }), { indent_size: 4 }) + "\n";
};

Bindings.prototype.getCargoCommand = function getCargoCommand() {
  var command, args;
  if (this.multirust) {
    command = "multirust";
    args = ["run", this.multirust, "cargo", "rustc"];
  } else {
    command = "cargo";
    args = ["rustc"];
  }
  if (this.mode === 'release') {
    args.push("--release");
  }
  args = args.concat(["--", "--crate-type", "staticlib"]);
  return {
    command: command,
    args: args
  };
};

Bindings.prototype.getStaticLibPath = function getStaticLibPath() {
  // FIXME: this is OSX-specific
  return "target/" + this.mode + "/lib" + this.name + ".a";
};

Bindings.prototype.getDynamicLibPath = function getDynamicLibPath() {
  // FIXME: this is OSX-specific
  return "target/" + this.mode + "/lib" + this.name + ".dylib";
};

Bindings.prototype.getBuildDynamicLibCommand = function getBuildDynamicLibCommand() {
  // FIXME: this is OSX-specific
  return {
    command: "gcc",
    args: ["-dynamiclib",
           "-Wl,-undefined,dynamic_lookup",
           "-Wl,-force_load," + this.getStaticLibPath(),
           "-o",
           this.getDynamicLibPath()]
  };
};

Bindings.prototype.getNodeGypPath = function getNodeGypPath() {
  // FIXME: not robust to dedupe
  return path.resolve(this.root, 'node_modules/rust-bindings/node_modules/.bin/node-gyp');
};

Bindings.prototype.getNanPath = function getNanPath() {
  // FIXME: not robust to dedupe
  return path.resolve(this.root, 'node_modules/rust-bindings/node_modules/nan');
};

Bindings.prototype.build = function build() {
  this.log("building static library");
  this.run(this.getCargoCommand());
  this.log("building dynamic library");
  this.run(this.getBuildDynamicLibCommand());
  this.log("generating boilerplate");
  fs.writeFileSync(path.resolve(this.root, "addon.cc"), this.getAddonSource());
  fs.writeFileSync(path.resolve(this.root, "binding.gyp"), this.getGypSource());
  this.log("building addon");
  this.run({ command: this.getNodeGypPath(), args: ["configure"] });
  this.run({ command: this.getNodeGypPath(), args: ["build"] });
  this.log("addon successfully built");
};

Bindings.prototype.getAddonPath = function getAddonPath() {
  return path.resolve(this.root, "build/" + this.mode[0].toUpperCase() + this.mode.substring(1) + "/" + this.name + ".node");
};

Bindings.prototype.log = function(msg) {
  // FIXME: use verbose flag to control this
  console.log(msg);
};

Bindings.prototype.run = function run(command) {
  this.log([command.command].concat(command.args).join(" "));
  // FIXME: stdio configuration should depend on verbose flag
  var result = child.spawnSync(command.command, command.args, { cwd: this.root, stdio: 'inherit' });
  if (result.status !== 0) {
    process.exit(result.status);
  }
};

Bindings.prototype.fileExists = function fileExists(filename) {
  try {
    return fs.statSync(filename).isFile();
  } catch (e) {
    return false;
  }
};

module.exports = exports = function rustBindings(opts) {
  var b = new Bindings(opts);

  var addonPath = b.getAddonPath();
  if (!b.fileExists(addonPath)) {
    b.build();
  }
  return require(addonPath);
};

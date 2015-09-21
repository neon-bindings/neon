var child = require('child_process');

function Builder(project) {
  this.project = project;
  this.mode = 'release';
  this.multirust = 'nightly';
}

Builder.prototype.log = function(msg) {
  // FIXME: use verbose flag to control this
  console.log(msg);
};

Builder.prototype.run = function run(command) {
  // FIXME: should log `[command.command].concat(command.args).join(" ")` somewhere
  // FIXME: should capture output and log it somewhere
  var result = child.spawnSync(command.command, command.args, { cwd: this.root, stdio: 'ignore' });
  if (result.status !== 0) {
    process.exit(result.status);
  }
};

Builder.prototype.getCargoCommand = function getCargoCommand() {
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

Builder.prototype.getStaticLibPath = function getStaticLibPath() {
  // FIXME: this is OSX-specific
  return "target/" + this.mode + "/lib" + this.project.libName + ".a";
};

Builder.prototype.getDynamicLibPath = function getDynamicLibPath() {
  // FIXME: this is OSX-specific
  return "target/" + this.mode + "/lib" + this.project.libName + ".dylib";
};

Builder.prototype.getBuildDynamicLibCommand = function getBuildDynamicLibCommand() {
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


module.exports = exports = function(project) {
  var builder = new Builder(project);
  builder.run(builder.getCargoCommand());
  builder.run(builder.getBuildDynamicLibCommand());
  console.log(builder.getDynamicLibPath());
};

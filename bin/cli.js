#!/usr/bin/env node

var bindings = require('bindings');

function getRoot() {
  return bindings.getRoot(bindings.getFileName(__filename));
}

if (process.argv.length !== 3) {
  printUsage();
  process.exit(1);
}

switch (process.argv[2]) {
case 'include':
  require('nan');
  break;

case 'generate':
  var root = getRoot();
  var manifest = root + "/Cargo.toml";
  var project = require('../lib/project.js')(root, manifest);
  var generate = require('../lib/generate.js')(project);
  generate.addon();
  generate.gyp();
  break;

case 'build':
  var Project = require('../lib/project.js');
  var root = getRoot();
  var manifest = root + "/Cargo.toml";
  var project = new Project(root, manifest);

  require('../lib/build.js')(project);
}

function printUsage() {
  console.log("rust-bindings generate");
  console.log("  generate build manifest and C++ wrapper");
  console.log("  (run by developer of Rust module)");
  console.log();
  console.log("rust-bindings build");
  console.log("  build the dynamic library and print its path");
  console.log("  (run by client of Rust module)");
  console.log();
  console.log("rust-bindings include");
  console.log("  print the path to the C++ include directory");
}

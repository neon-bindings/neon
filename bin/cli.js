#!/usr/bin/env node

var bindings = require('bindings');
var path = require('path');
var fs = require('fs');
var minimist = require('minimist');
var project = require('../lib/project.js');

function fileExists(filename) {
  try {
    return fs.statSync(filename).isFile();
  } catch (e) {
    return false;
  }
}

if (process.argv.length < 3) {
  printUsage();
  process.exit(1);
}

var command = process.argv[2];
var args = minimist(process.argv.slice(3));
var pwd = process.cwd();

switch (command) {
case 'include-path':
  require('nan');
  break;

case 'build':
  // argument validation
  if (args.rust === true || args.r === true) {
    printUsage();
    process.exit(1);
  }
  var build = require('../lib/build.js')(project(pwd),
                                         args.rust || args.r || 'nightly', // ISSUE: https://github.com/dherman/rust-bindings/issues/2
                                         (args.debug || args.d) ? 'debug' : 'release');
  if (build.isStale()) {
    build.run();
  }
  break;

case 'generate':
  // argument validation
  if (args._.length === 0) {
    printUsage();
    process.exit(1);
  }
  var addon = require('../lib/addon.js')(project(pwd));
  addon.generate(args._[0]);
  break;
}

// FIXME: allow the build command to take a --manifest path and a --name string

function printUsage() {
  console.log("Usage:");
  console.log();
  console.log("  rust-bindings build [--rust|-r nightly|stable|default] [--debug|-d]");
  console.log("    build the native module");
  console.log();
  console.log("  rust-bindings generate filename");
  console.log("    generate the native module's C++ wrapper at filename");
  console.log();
  console.log("  rust-bindings include-path");
  console.log("    print the path to the C++ include directory");
}

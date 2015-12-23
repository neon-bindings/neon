#!/usr/bin/env node

var path = require('path');
var fs = require('fs');
var minimist = require('minimist');
var project = require('neon-bridge').project;

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
case 'help':
  printUsage();
  break;

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
                                         args.rust || args.r || 'default',
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

case 'new':
  if (args._.length !== 1) {
    printUsage();
    process.exit(1);
  }
  var create = require('../lib/create.js');
  create(pwd, args._[0]);
  break;
}

// FIXME: allow the build command to take a --manifest path and a --name string

function printUsage() {
  console.log("Usage:");
  console.log();
  console.log("  neon new name");
  console.log("    create a new Neon project");
  console.log();
  console.log("  neon build [--rust|-r nightly|stable|default] [--debug|-d]");
  console.log("    build a Neon native module");
  console.log();
  console.log("  neon help");
  console.log("    print this usage information");
  console.log();
}

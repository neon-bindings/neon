#!/usr/bin/env node

var path = require('path')
var minimist = require('minimist');
var pkg = require(path.resolve(__dirname, '../package.json'));

if (process.argv.length < 3) {
  printUsage();
  process.exit(1);
}

var command = process.argv[2];
var args = minimist(process.argv.slice(3));
var pwd = process.cwd();

switch (command) {
case 'version':
  console.log(pkg.version)
  break;
case 'help':
  printUsage();
  break;

case 'new':
  if (args._.length !== 1) {
    printUsage();
    process.exit(1);
  }
  var create = require('../lib/create.js').default;
  create(pwd, args._[0], args.rust || args.r || 'default');
  break;
}

function printUsage() {
  console.log();
  console.log("Usage:");
  console.log();
  console.log("  neon new <name> [--rust|-r nightly|stable|default]");
  console.log("    create a new Neon project");
  console.log();
  console.log("  neon help");
  console.log("    print this usage information");
  console.log();
  console.log("  neon version");
  console.log("    print neon-cli version");
  console.log();
  console.log("neon-cli@" + pkg.version + " " + path.dirname(__dirname));
}

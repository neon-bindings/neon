#!/usr/bin/env node

var minimist = require('minimist');

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
  console.log("Usage:");
  console.log();
  console.log("  neon new <name> [--rust|-r nightly|stable|default]");
  console.log("    create a new Neon project");
  console.log();
  console.log("  neon help");
  console.log("    print this usage information");
  console.log();
}

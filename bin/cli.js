#!/usr/bin/env node

var CLI = require('../lib/cli.js').default;
var cli = new CLI(process.argv, process.cwd());
cli.exec()
   .then(function() { process.exit(0); })
   .catch(function() { process.exit(1); });

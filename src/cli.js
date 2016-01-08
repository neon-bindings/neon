import path from 'path';
import metadata from '../package.json';
import minimist from 'minimist';
import neon_new from './ops/neon_new';
import neon_build from './ops/neon_build';
import * as style from './ops/style';

function printUsage() {
  console.log();
  console.log("Usage:");
  console.log();
  console.log("  neon new [@<scope>/]<name> [--rust|-r nightly|stable|default]");
  console.log("    create a new Neon project");
  console.log();
  console.log("  neon version");
  console.log("    print neon-cli version");
  console.log();
  console.log("  neon help");
  console.log("    print this usage information");
  console.log();
  console.log("neon-cli@" + metadata.version + " " + path.dirname(__dirname));
}

const SUBCOMMANDS = {
  'version': function() {
    console.log(metadata.version);
  },

  'help': function() {
    printUsage();
  },

  'new': function() {
    if (this.args._.length !== 1) {
      printUsage();
      console.log();
      throw new Error(this.args._.length === 0 ? "You must specify a project name." : "Too many arguments.");
    }
    return neon_new(this.cwd, this.args._[0], this.args.rust || this.args.r || 'default');
  },

  'build': function() {
    if (this.args._.length > 0) {
      printUsage();
      console.log();
      throw new Error("Too many arguments.");
    }
    return neon_build(this.cwd,
                      this.args.rust || this.args.r || 'default',
                      this.args.debug || this.args.d ? 'debug' : 'release');
  }
};

export default class CLI {
  constructor(argv, cwd) {
    this.command = argv[2];
    this.args = minimist(argv.slice(3));
    this.cwd = cwd;
  }

  async exec() {
    try {
      if (!this.command) {
        printUsage();
        throw null;
      }
      if (!SUBCOMMANDS.hasOwnProperty(this.command)) {
        printUsage();
        console.log();
        throw new Error("'" + this.command + "' is not a neon command.");
      }
      await SUBCOMMANDS[this.command].call(this);
    } catch (e) {
      if (e) {
        console.log(style.error(e.message));
      }
      throw e;
    }
  }
}

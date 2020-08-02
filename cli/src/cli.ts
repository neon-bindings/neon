import path from 'path';
import neon_new from './ops/neon_new';
import neon_build from './ops/neon_build';
import neon_clean from './ops/neon_clean';
import * as style from './style';
import cliCommands = require('command-line-commands');
import cliArgs = require('command-line-args');
import cliUsage = require('command-line-usage');
import log from './log';
import { setup as setupLogging } from './log';
import * as JSON from 'ts-typed-json';
import { Toolchain } from './rust';

let metadata = JSON.loadSync(path.resolve(__dirname, '..', 'package.json'));

function commandUsage(command: string) {
  if (!spec[command]) {
    let e = new Error();
    (e as any).command = command;
    e.name = 'INVALID_COMMAND';
    throw e;
  }
  console.error(cliUsage(spec[command].usage));
}

function logIf(multiple: boolean, action: string, cwd: string, module: string) {
  if (multiple) {
    log(action + " Neon package at " + (path.relative(cwd, module) || "."));
  }
}

function parseModules(cwd: string, names: string[], paths: boolean) {
  let modules = names.length
      ? names.map(m => paths ? path.resolve(cwd, m)
                             : path.resolve(cwd, 'node_modules', m))
      : [cwd];

  return {
    modules,
    multiple: modules.length > 1
  };
}

type Action = (this: CLI, options: Record<string, unknown>, usage: string) => void;

type Command = {
  args: cliArgs.OptionDefinition[],
  usage: cliUsage.Sections,
  action: Action
};

type Spec = Record<string, Command>;

const spec: Spec = {

  null: {
    args: [{ name: "version", alias: "v", type: Boolean },
           { name: "help", alias: "h", type: String, defaultValue: null }],
    usage: [{
      header: "Neon",
      content: "Neon is a tool for building native Node.js modules with Rust."
    }, {
      header: "Synopsis",
      content: "$ neon [options] <command>"
    }, {
      header: "Command List",
      content: [{ name: "new", summary: "Create a new Neon project." },
                { name: "build", summary: "(Re)build a Neon project." },
                { name: "clean", summary: "Remove build artifacts from a Neon project." },
                { name: "version", summary: "Display the Neon version." },
                { name: "help", summary: "Display help information about Neon." }]
    }],
    action: function(options, usage) {
      if (options.version && options.help === undefined) {
        spec.version.action.call(this, options);
      } else if (options.help !== undefined) {
        commandUsage(options.help as string);
      } else {
        console.error(usage);
      }
    }
  },

  help: {
    args: [{ name: "command", type: String, defaultOption: true },
           { name: "help", alias: "h", type: Boolean }],
    usage: [{
      header: "neon help",
      content: "Get help about a Neon command"
    }, {
      header: "Synopsis",
      content: "$ neon help [command]"
    }],
    action: function(options) {
      if (options && options.command) {
        commandUsage(options.command as string);
      } else if (options && options.help) {
        commandUsage('help');
      } else {
        console.error(cliUsage(spec.null.usage));
      }
    }
  },

  new: {
    args: [{ name: "name", type: String, defaultOption: true },
           { name: "neon", alias: "n", type: String },
           { name: "features", alias: "f", type: String },
           { name: "no-default-features", type: Boolean },
           { name: "help", alias: "h", type: Boolean }],
    usage: [{
      header: "neon new",
      content: "Create a new Neon project."
    }, {
      header: "Synopsis",
      content: "$ neon new [options] [@<scope>/]<name>"
    }, {
      header: "Options",
      optionList: [{
        name: "neon",
        alias: "n",
        type: String,
        description: "Specify a semver version of Neon or path to a local Neon repository."
      }, {
        name: "features",
        alias: "f",
        type: String,
        description: "Space-separated list of experimental Neon features to enable."
      }, {
        name: "no-default-features",
        type: Boolean,
        description: "Do not activate the `default` Neon feature."
      }]
    }],
    action: function(options) {
      if (options.help) {
        commandUsage('new');
      } else if (!options.name) {
        console.error(cliUsage(spec.new.usage));
      } else {
        return neon_new(this.cwd,
                        options.name as string,
                        (options.neon || null) as (string | null),
                        (options.features || null) as (string | null),
                        !!options['no-default-features']);
      }
      return;
    }
  },

  build: {
    args: [{ name: "release", alias: "r", type: Boolean },
           { name: "path", alias: "p", type: Boolean },
           { name: "modules", type: String, multiple: true, defaultOption: true },
           { name: "help", alias: "h", type: Boolean }],
    usage: [{
      header: "neon build",
      content: "(Re)build a Neon project."
    }, {
      header: "Synopsis",
      content: ["$ neon build [options]",
                "$ neon build [options] {underline module} ..."]
    }, {
      header: "Options",
      optionList: [{
        name: "release",
        alias: "r",
        type: Boolean,
        description: "Release build."
      }, {
        name: "path",
        alias: "p",
        type: Boolean,
        description: "Specify modules by path instead of name."
      }]
    }],
    action: async function(options) {
      if (options.help) {
        commandUsage('build');
        return;
      }

      let { modules, multiple } = parseModules(this.cwd,
                                               (options.modules || []) as string[],
                                               !!options.path);

      for (let module of modules) {
        logIf(multiple, "building", this.cwd, module);

        await neon_build(module, this.toolchain, !!options.release);
      }
    }
  },

  clean: {
    args: [{ name: "path", alias: "p", type: Boolean },
           { name: "modules", type: String, multiple: true, defaultOption: true },
           { name: "help", alias: "h", type: Boolean }],
    usage: [{
      header: "neon clean",
      content: "Remove build artifacts from a Neon project."
    }, {
      header: "Synopsis",
      content: ["$ neon clean [options]",
                "$ neon clean [options] {underline module} ..."]
    }, {
      header: "Options",
      optionList: [{
        name: "path",
        alias: "p",
        type: Boolean,
        description: "Specify modules by path instead of name."
      }]
    }],
    action: async function(options) {
      if (options.help) {
        commandUsage('clean');
        return;
      }

      let { modules, multiple } = parseModules(this.cwd,
                                               (options.modules || []) as string[],
                                               !!options.path);

      for (let module of modules) {
        logIf(multiple, "cleaning", this.cwd, module);

        await neon_clean(module);
      }
    }
  },

  version: {
    args: [{ name: "help", alias: "h", type: Boolean }],
    usage: [{
      header: "neon version",
      content: "Display the Neon version."
    }, {
      header: "Synopsis",
      content: "$ neon version"
    }],
    action: function(options) {
      if (options.help) {
        commandUsage('version');
        return;
      }

      console.log((JSON.asObject(metadata)).version);
    }
  }

};

export default class CLI {
  readonly toolchain: Toolchain;
  readonly argv: string[];
  readonly cwd: string;

  constructor(argv: string[], cwd: string) {
    // Check for a toolchain argument in the style of Rust tools (e.g., `neon +nightly build`).
    if (argv.length > 2 && argv[2].trim().startsWith('+')) {
      this.toolchain = argv[2].substring(1).trim() as Toolchain;
      this.argv = argv.slice(3);
    } else {
      this.toolchain = 'default';
      this.argv = argv.slice(2);
    }
    this.cwd = cwd;
  }

  async exec() {
    setupLogging(msg => { console.log(style.info(msg)); });

    let parsed;

    try {
      parsed = cliCommands([ null, 'help', 'new', 'build', 'clean', 'version' ], this.argv);
    } catch (e) {
      spec.help.action.call(this);

      switch (e.name) {
        case 'INVALID_COMMAND':
          console.error(style.error("No manual entry for `neon " + e.command + "`"));
          break;

        default:
          console.error(style.error(e.message));
          break;
      }

      process.exit(1);
    }

    try {
      let { command, argv } = parsed;
      await spec[command].action.call(this,
                                      cliArgs(spec[command].args, { argv }),
                                      cliUsage(spec[command].usage));
    } catch (e) {
      console.error(style.error(e.message));
      console.error();
      console.error(e.stack);
      throw e;
    }
  }
}

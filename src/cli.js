import path from 'path';
import metadata from '../package.json';
import neon_new from './ops/neon_new';
import neon_build from './ops/neon_build';
import neon_clean from './ops/neon_clean';
import * as style from './ops/style';
import parseCommands from 'command-line-commands';
import parseArgs from 'command-line-args';
import parseUsage from 'command-line-usage';
import log from './log';
import { setup as setupLogging } from './log';

function channel(value) {
  if (!['default', 'nightly', 'beta', 'stable'].indexOf(value) > -1) {
    throw new Error("Expected one of 'default', 'nightly', 'beta', or 'stable', got '" + value + "'");
  }
  return value;
}

function profile(value) {
  if (!['debug', 'release', 'all'].indexOf(value) > -1) {
    throw new Error("Expected one of 'debug', 'release', or 'all', got '" + value + "'");
  }
  return value;
}

function commandUsage(command) {
  if (!spec[command]) {
    let e = new Error();
    e.command = command;
    e.name = 'INVALID_COMMAND';
    throw e;
  }
  console.error(parseUsage(spec[command].usage));
}

function logIf(multiple, action, cwd, module) {
  if (multiple) {
    log(action + " Neon package at " + (path.relative(cwd, module) || "."));
  }
}

function parseModules(cwd, names, paths) {
  let modules = names
      ? names.map(m => paths ? path.resolve(cwd, m)
                             : path.resolve(cwd, 'node_modules', m))
      : [cwd];

  return {
    modules,
    multiple: modules.length > 1
  };
}

const spec = {

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
        commandUsage(options.help);
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
        commandUsage(options.command);
      } else if (options && options.help) {
        commandUsage('help');
      } else {
        console.error(parseUsage(spec.null.usage));
      }
    }
  },

  new: {
    args: [{ name: "name", type: String, defaultOption: true },
           { name: "help", alias: "h", type: Boolean }],
    usage: [{
      header: "neon new",
      content: "Create a new Neon project."
    }, {
      header: "Synopsis",
      content: "$ neon new [@<scope>/]<name>"
    }],
    action: function(options) {
      if (options.help) {
        commandUsage('new');
        return;
      }

      return neon_new(this.cwd, options.name);
    }
  },

  build: {
    args: [{ name: "debug", alias: "d", type: Boolean },
           { name: "path", alias: "p", type: Boolean },
           { name: "rust", alias: "r", type: channel, defaultValue: "default" },
           { name: "modules", type: String, multiple: true, defaultOption: true },
           { name: "node_module_version", type: Number },
           { name: "help", alias: "h", type: Boolean }],
    usage: [{
      header: "neon build",
      content: "(Re)build a Neon project."
    }, {
      header: "Synopsis",
      content: ["$ neon build [options]",
                "$ neon build [options] [underline]{module} ..."]
    }, {
      header: "Options",
      optionList: [{
        name: "rust",
        alias: "r",
        type: channel,
        description: "Rust channel (default, nightly, beta, or stable). [default: default]"
      }, {
        name: "debug",
        alias: "d",
        type: Boolean,
        description: "Debug build."
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

      let { modules, multiple } = parseModules(this.cwd, options.modules, options.path);

      for (let module of modules) {
        logIf(multiple, "building", this.cwd, module);

        await neon_build(module, options.rust, !options.debug, options.node_module_version);
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
                "$ neon clean [options] [underline]{module} ..."]
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

      let { modules, multiple } = parseModules(this.cwd, options.modules, options.path);

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

      console.log(metadata.version);
    }
  }

};

export default class CLI {
  constructor(argv, cwd) {
    this.argv = argv.slice(2);
    this.cwd = cwd;
  }

  async exec() {
    setupLogging(msg => { console.log(style.info(msg)); });

    try {
      let { command, argv } = parseCommands([ null, 'help', 'new', 'build', 'clean', 'version' ], this.argv);

      await spec[command].action.call(this,
                                      parseArgs(spec[command].args, { argv }),
                                      parseUsage(spec[command].usage));
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
    }
  }
}

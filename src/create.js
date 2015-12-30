import fs from 'fs';
import path from 'path';
import handlebars from 'handlebars';
import mkdirp from 'mkdirp';
import inquirer from 'inquirer';
import semver from 'semver';
import bridge from 'neon-bridge';
import gitconfig from 'git-config';
import validateLicense from 'validate-npm-package-license';
import validateName from 'validate-npm-package-name';
import chalk from 'chalk';

const ROOT_DIR = path.resolve(__dirname, "..");
const TEMPLATES_DIR = path.resolve(ROOT_DIR, "templates");

const NEON_CLI_VERSION = JSON.parse(fs.readFileSync(path.resolve(ROOT_DIR, "package.json"), 'utf8')).version;
const NEON_BRIDGE_VERSION = bridge.version;

function compile(filename) {
  return handlebars.compile(fs.readFileSync(path.resolve(TEMPLATES_DIR, filename), 'utf8'), { noEscape: true });
}

const GITIGNORE_TEMPLATE = compile(".gitignore.hbs");
const CARGO_TEMPLATE = compile("Cargo.toml.hbs");
const NPM_TEMPLATE = compile("package.json.hbs");
const INDEXJS_TEMPLATE = compile("index.js.hbs");
const LIBRS_TEMPLATE = compile("lib.rs.hbs");
const README_TEMPLATE = compile("README.md.hbs");
const CPP_TEMPLATE = compile("binding.cc.hbs");
const GYP_TEMPLATE = compile("binding.gyp.hbs");

function die(err) {
  console.log(err);
  process.exit(1);
}

function guessAuthor() {
  try {
    let config = gitconfig.sync();
    if (config.user.name) {
      return {
        author: config.user.name,
        email: config.user.email
      };
    }
  } catch (e) {
    return {
      author: process.env.USER || process.env.USERNAME,
      email: undefined
    };
  }
}

let style = {
  project: chalk.cyan.bold,
  command: chalk.green.bold,
  path: chalk.cyan
};

export default function wizard(pwd, name, toolchain) {
  let its = validateName(name);
  if (!its.validForNewPackages) {
    let errors = (its.errors || []).concat(its.warnings || []);
    console.log("Sorry, " + errors.join(" and ") + ".");
    process.exit(1);
  }

  // check for a scoped name
  let scoped = name.match(/@([^\/]+)\/(.*)/);
  let [, scope, local] = scoped || [, null, name];

  console.log("This utility will walk you through creating the " + style.project(name) + " Neon project.");
  console.log("It only covers the most common items, and tries to guess sensible defaults.");
  console.log();
  console.log("Press ^C at any time to quit.");

  let root = path.resolve(pwd, local);
  let guess = guessAuthor();

  inquirer.prompt([
    {
      type: 'input',
      name: 'version',
      message: "version",
      default: "0.1.0",
      validate: function (input) {
        if (semver.valid(input)) {
          return true;
        }
        return "Invalid version: " + input;
      }
    },
    { type: 'input', name: 'description', message: "description"                               },
    { type: 'input', name: 'node',        message: "node entry point", default: "lib/index.js" },
    { type: 'input', name: 'git',         message: "git repository"                            },
    { type: 'input', name: 'author',      message: "author",           default: guess.author   },
    { type: 'input', name: 'email',       message: "email",            default: guess.email    },
    {
      type: 'input',
      name: 'license',
      message: "license",
      default: "MIT",
      validate: function (input) {
        let its = validateLicense(input);
        if (its.validForNewPackages) {
          return true;
        }
        let errors = (its.errors || []).concat(its.warnings || []);
        return "Sorry, " + errors.join(" and ") + ".";
      }
    }
  ], function(answers) {
    answers.name = {
      npm: {
        full: name,
        scope: scope,
        local: local
      },
      cargo: {
        external: local,
        internal: local.replace(/-/g, "_")
      }
    };
    let ctx = {
      project: answers,
      "neon-cli": {
        major: semver.major(NEON_CLI_VERSION),
        minor: semver.minor(NEON_CLI_VERSION),
        patch: semver.patch(NEON_CLI_VERSION)
      },
      "neon-bridge": {
        major: semver.major(NEON_BRIDGE_VERSION),
        minor: semver.minor(NEON_BRIDGE_VERSION),
        patch: semver.patch(NEON_BRIDGE_VERSION)
      },
      build: {
        cargo: {
          cmd: toolchain === 'default'
             ? []
             : ["multirust", "run", toolchain]
        }
      }
    };

    let lib = path.resolve(root, path.dirname(answers.node));
    let src = path.resolve(root, "src");

    mkdirp(lib, function(err) {
      if (err) die(err);
      mkdirp(src, function(err) {
        if (err) die(err);
        fs.writeFileSync(path.resolve(root, ".gitignore"), GITIGNORE_TEMPLATE(ctx), { flag: 'wx' });
        fs.writeFileSync(path.resolve(root, "package.json"), NPM_TEMPLATE(ctx), { flag: 'wx' });
        fs.writeFileSync(path.resolve(root, "Cargo.toml"), CARGO_TEMPLATE(ctx), { flag: 'wx' });
        fs.writeFileSync(path.resolve(root, "README.md"), README_TEMPLATE(ctx), { flag: 'wx' });
        fs.writeFileSync(path.resolve(root, answers.node), INDEXJS_TEMPLATE(ctx), { flag: 'wx' });
        fs.writeFileSync(path.resolve(root, "binding.gyp"), GYP_TEMPLATE(ctx), { flag: 'wx' });
        fs.writeFileSync(path.resolve(src, "binding.cc"), CPP_TEMPLATE(ctx), { flag: 'wx' });
        fs.writeFileSync(path.resolve(src, "lib.rs"), LIBRS_TEMPLATE(ctx), { flag: 'wx' });

        let relativeRoot = path.relative(pwd, root);
        let relativeNode = path.relative(pwd, path.resolve(root, answers.node));
        let relativeRust = path.relative(pwd, path.resolve(root, src + "/lib.rs"));

        console.log();
        console.log("Woo-hoo! Your Neon project has been created in: " + style.path(relativeRoot));
        console.log();
        console.log("The main Node entry point is at: " + style.path(relativeNode));
        console.log("The main Rust entry point is at: " + style.path(relativeRust));
        console.log();
        console.log("To build your project, just run " + style.command("npm install") + " from within the " + style.path(relativeRoot) + " directory.");
        console.log("Then you can test it out with " + style.command("node -e 'require(\"./\")'") + ".");
        console.log();
        console.log("Happy hacking!");
      });
    });
  });
};

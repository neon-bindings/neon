import { readFile, writeFile, mkdirs } from '../async/fs';
import { prompt } from '../async/inquirer';
import gitconfig from '../async/git-config';
import path from 'path';
import handlebars from 'handlebars';
import semver from 'semver';
import validateLicense from 'validate-npm-package-license';
import validateName from 'validate-npm-package-name';
import * as style from './style';

const ROOT_DIR = path.resolve(__dirname, '..', '..');
const TEMPLATES_DIR = path.resolve(ROOT_DIR, 'templates');

const NEON_CLI_VERSION = (async function() {
  return JSON.parse(await readFile(path.resolve(ROOT_DIR, 'package.json'), 'utf8')).version;
})();

async function compile(filename) {
  return handlebars.compile(await readFile(path.resolve(TEMPLATES_DIR, filename), 'utf8'), { noEscape: true });
}

const GITIGNORE_TEMPLATE = compile('.gitignore.hbs');
const CARGO_TEMPLATE     = compile('Cargo.toml.hbs');
const NPM_TEMPLATE       = compile('package.json.hbs');
const INDEXJS_TEMPLATE   = compile('index.js.hbs');
const LIBRS_TEMPLATE     = compile('lib.rs.hbs');
const README_TEMPLATE    = compile('README.md.hbs');
const BUILDRS_TEMPLATE   = compile('build.rs.hbs');

async function guessAuthor() {
  let author = {
    name: process.env.USER || process.env.USERNAME,
    email: undefined
  };
  try {
    let config = await gitconfig();
    if (config.user.name) {
      author.name = config.user.name;
    }
    if (config.user.email) {
      author.email = config.user.email;
    }
    return author;
  } catch (e) {
    return author;
  }
}

export default async function wizard(pwd, name) {
  let its = validateName(name);
  if (!its.validForNewPackages) {
    let errors = (its.errors || []).concat(its.warnings || []);
    throw new Error("Sorry, " + errors.join(" and ") + ".");
  }

  // check for a scoped name
  let scoped = name.match(/@([^\/]+)\/(.*)/);
  let [, scope, local] = scoped || [, null, name];

  console.log("This utility will walk you through creating the " + style.project(name) + " Neon project.");
  console.log("It only covers the most common items, and tries to guess sensible defaults.");
  console.log();
  console.log("Press ^C at any time to quit.");

  let root = path.resolve(pwd, local);
  let guess = await guessAuthor();

  let answers = await prompt([
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
    { type: 'input', name: 'author',      message: "author",           default: guess.name     },
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
  ]);

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
  let version = await NEON_CLI_VERSION;
  let ctx = {
    project: answers,
    "neon-cli": {
      major: semver.major(version),
      minor: semver.minor(version),
      patch: semver.patch(version)
    }
  };

  let lib = path.resolve(root, path.dirname(answers.node));
  let native_ = path.resolve(root, 'native');
  let src = path.resolve(native_, 'src');

  await mkdirs(lib);
  await mkdirs(src);

  await writeFile(path.resolve(root,    '.gitignore'),   (await GITIGNORE_TEMPLATE)(ctx), { flag: 'wx' });
  await writeFile(path.resolve(root,    'package.json'), (await NPM_TEMPLATE)(ctx),       { flag: 'wx' });
  await writeFile(path.resolve(native_, 'Cargo.toml'),   (await CARGO_TEMPLATE)(ctx),     { flag: 'wx' });
  await writeFile(path.resolve(root,    'README.md'),    (await README_TEMPLATE)(ctx),    { flag: 'wx' });
  await writeFile(path.resolve(root,    answers.node),   (await INDEXJS_TEMPLATE)(ctx),   { flag: 'wx' });
  await writeFile(path.resolve(src,     'lib.rs'),       (await LIBRS_TEMPLATE)(ctx),     { flag: 'wx' });
  await writeFile(path.resolve(native_, 'build.rs'),     (await BUILDRS_TEMPLATE)(ctx),   { flag: 'wx' });

  let relativeRoot = path.relative(pwd, root);
  let relativeNode = path.relative(pwd, path.resolve(root, answers.node));
  let relativeRust = path.relative(pwd, path.resolve(src, 'lib.rs'));

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
};

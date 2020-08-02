import { mkdirSync, writeFileSync, promises as fsPromises } from 'fs';
import { prompt } from 'inquirer';
import path from 'path';
import handlebars from 'handlebars';
import semver from 'semver';
import * as style from '../style';
import validateLicense = require('validate-npm-package-license');
import validateName = require('validate-npm-package-name');
import * as JSON from 'ts-typed-json';
import gitconfig from 'git-config';

const { readFile, stat } = fsPromises;

const ROOT_DIR = path.resolve(__dirname, '..', '..');
const TEMPLATES_DIR = path.resolve(ROOT_DIR, 'templates');

const NEON_CLI_VERSION =
  JSON.asString(JSON.asObject(JSON.loadSync(path.resolve(ROOT_DIR, 'package.json'))).version);

async function compile(filename: string) {
  let source = await readFile(path.resolve(TEMPLATES_DIR, filename), {
    encoding: 'utf8'
  });
  return handlebars.compile(source, { noEscape: true });
}

const GITIGNORE_TEMPLATE = compile('.gitignore.hbs');
const CARGO_TEMPLATE     = compile('Cargo.toml.hbs');
const NPM_TEMPLATE       = compile('package.json.hbs');
const INDEXJS_TEMPLATE   = compile('index.js.hbs');
const LIBRS_TEMPLATE     = compile('lib.rs.hbs');
const README_TEMPLATE    = compile('README.md.hbs');
const BUILDRS_TEMPLATE   = compile('build.rs.hbs');

type Author = {
  name?: string,
  email?: string
};

async function guessAuthor() {
  let author: Author = {
    name: process.env.USER || process.env.USERNAME,
    email: undefined
  };
  try {
    let config = gitconfig.sync();
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

type NeonVersion = { type: "version" | "range" | "relative" | "absolute", value: string };

async function parseNeonVersion(flag: string | null) : Promise<NeonVersion> {
  if (!flag) {
    return { type: "version", value: NEON_CLI_VERSION };
  }

  if (semver.valid(flag)) {
    return { type: "version", value: flag };
  }

  if (semver.validRange(flag)) {
    return { type: "range", value: flag };
  }

  let stats = await stat(flag);

  if (!stats.isDirectory()) {
    throw new Error("Specified path to Neon is not a directory");
  }

  return { type: path.isAbsolute(flag) ? "absolute" : "relative", value: flag };
}

interface Answers {
  name: {
    npm: {
      full: string;
      scope: string | null;
      local: string;
    };
    cargo: {
      external: string;
      internal: string;
    };
  };
  description: string;
  git: string;
  author: string;
  node: string;
}

export default async function wizard(pwd: string, name: string, neon: string | null, features: string | null, noDefaultFeatures: boolean) {
  let its = validateName(name);
  if (!its.validForNewPackages) {
    let errors = (its.errors || []).concat(its.warnings || []);
    throw new Error("Sorry, " + errors.join(" and ") + ".");
  }

  // check for a scoped name
  let scoped = name.match(/@([^\/]+)\/(.*)/);
  let [, scope, local] = scoped ? (scoped as [string, string, string]) : [, null, name];

  console.log("This utility will walk you through creating the " + style.project(name) + " Neon project.");
  console.log("It only covers the most common items, and tries to guess sensible defaults.");
  console.log();
  console.log("Press ^C at any time to quit.");

  let root = path.resolve(pwd, local);
  let guess = await guessAuthor();

  let answers: Answers = await prompt([
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
        let errors = its.warnings || [];
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
  answers.description = escapeQuotes(answers.description);
  answers.git = encodeURI(answers.git);
  answers.author = escapeQuotes(answers.author);

  let neonVersion = await parseNeonVersion(neon);

  let simple = (neonVersion.type === 'version' || neonVersion.type === 'range')
    && !features
    && !noDefaultFeatures;

  let libs: {
    // In the common case, we can make the Cargo.toml manifest simple by just using
    // the semver specifier string for the `neon` and `neon-build` dependencies.
    simple: boolean,
    paths?: { neon: string, 'neon-build': string },
    version?: string,
    features?: Array<string>,
    noDefaultFeatures: boolean
  } = { simple, noDefaultFeatures };

  if (neonVersion.type === 'relative') {
    let neon = path.relative(path.join(name, 'native'), neonVersion.value);
    libs.paths = {
      neon: JSON.stringify(neon),
      'neon-build': JSON.stringify(path.join(neon, 'crates', 'neon-build'))
    };
  } else if (neonVersion.type === 'absolute') {
    libs.paths = {
      neon: JSON.stringify(neonVersion.value),
      'neon-build': JSON.stringify(path.resolve(neonVersion.value, 'crates', 'neon-build'))
    };
  } else {
    libs.version = JSON.stringify(neonVersion.value);
  }

  if (features) {
    libs.features = features.split(/\s+/).map(JSON.stringify);
  }

  let cli = JSON.stringify(neonVersion.type === 'version'
    ? "^" + neonVersion.value
    : neonVersion.type === 'relative'
    ? "file:" + path.join(path.relative(name, neonVersion.value), 'cli')
    : neonVersion.type === 'absolute'
    ? "file:" + path.resolve(neonVersion.value, 'cli')
    : neonVersion.value);

  let ctx = {
    project: answers,
    neon: { cli, libs }
  };

  let lib = path.resolve(root, path.dirname(answers.node));
  let native_ = path.resolve(root, 'native');
  let src = path.resolve(native_, 'src');

  mkdirSync(lib, { recursive: true });
  mkdirSync(src, { recursive: true });

  writeFileSync(path.resolve(root,    '.gitignore'),   (await GITIGNORE_TEMPLATE)(ctx), { flag: 'wx' });
  writeFileSync(path.resolve(root,    'package.json'), (await NPM_TEMPLATE)(ctx),       { flag: 'wx' });
  writeFileSync(path.resolve(native_, 'Cargo.toml'),   (await CARGO_TEMPLATE)(ctx),     { flag: 'wx' });
  writeFileSync(path.resolve(root,    'README.md'),    (await README_TEMPLATE)(ctx),    { flag: 'wx' });
  writeFileSync(path.resolve(root,    answers.node),   (await INDEXJS_TEMPLATE)(ctx),   { flag: 'wx' });
  writeFileSync(path.resolve(src,     'lib.rs'),       (await LIBRS_TEMPLATE)(ctx),     { flag: 'wx' });
  writeFileSync(path.resolve(native_, 'build.rs'),     (await BUILDRS_TEMPLATE)(ctx),   { flag: 'wx' });

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

function escapeQuotes(str: string): string {
  return str.replace(/"/g, '\\"');
}

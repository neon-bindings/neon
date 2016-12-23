import { remove, copy, readFile } from '../async/fs';
import { spawn } from '../async/child_process';
import path from 'path';
import handlebars from 'handlebars';
import * as style from './style';
import { removeSync, copySync } from 'fs-extra';
import TOML from 'toml';
import clone from 'shallow-copy';

const TEMPLATES_DIR = path.resolve(__dirname, '..', 'templates');

const LIB_PREFIX = {
  'darwin':  "lib",
  'freebsd': "lib",
  'linux':   "lib",
  'sunos':   "lib",
  'win32':   ""
};

const LIB_SUFFIX = {
  'darwin':  ".dylib",
  'freebsd': ".so",
  'linux':   ".so",
  'sunos':   ".so",
  'win32':   ".dll"
};

function cargo(toolchain, configuration, nodeModuleVersion) {
  let macos = process.platform === 'darwin';

  let [command, prefix] = toolchain === 'default'
                        ? ["cargo", []]
                        : ["multirust", ["run", toolchain]];

  let args = prefix.concat(macos ? 'rustc' : 'build',
                           configuration === 'release' ? ["--release"] : [],
                           macos ? ["--", "-C", "link-args=-Wl,-undefined,dynamic_lookup"] : []);

  console.log(style.info([command].concat(args).join(" ")));

  return spawn(command, args, { cwd: 'native', stdio: 'inherit' });
}

async function main(name, configuration) {
  let pp = process.platform;
  let dylib = path.resolve('native', 'target', configuration, LIB_PREFIX[pp] + name + LIB_SUFFIX[pp]);
  let index = path.resolve('native', 'index.node');

  console.log(style.info("generating native" + path.sep + "index.node"));

  await remove(index);
  await copy(dylib, index);
}

export default async function neon_build(pwd, toolchain, configuration, nodeModuleVersion) {
  // 1. Read the Cargo metadata.
  let metadata = TOML.parse(await readFile(path.resolve('native', 'Cargo.toml'), 'utf8'));

  if (!metadata.lib.name) {
    throw new Error("Cargo.toml does not contain a [lib] section with a 'name' field");
  }

  console.log(style.info("running cargo"));

  // 2. Build the binary.
  if ((await cargo(toolchain, configuration, nodeModuleVersion)) !== 0) {
    throw new Error("cargo build failed");
  }

  // 3. Copy the dylib into the main index.node file.
  await main(metadata.lib.name, configuration);
}

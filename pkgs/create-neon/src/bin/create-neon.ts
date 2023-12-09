#!/usr/bin/env node

import { promises as fs } from "fs";
import * as path from "path";
import die from "../die.js";
import Package from "../package.js";
import { VERSIONS } from '../versions.js';
import expand from "../expand.js";
import chalk from 'chalk';

function pink(text: string): string {
  return chalk.bold.hex('#e75480')(text);
}

function blue(text: string): string {
  return chalk.bold.cyanBright(text);
}

const TEMPLATES: Record<string, string> = {
  ".gitignore.hbs": ".gitignore",
  "Cargo.toml.hbs": "Cargo.toml",
  "README.md.hbs": "README.md",
  "lib.rs.hbs": path.join("src", "lib.rs"),
};

async function main(name: string) {
  let tmpFolderName: string = "";

  try {
    // pretty lightweight way to check both that folder doesn't exist and
    // that the user has write permissions.
    await fs.mkdir(name);
    await fs.rmdir(name);

    tmpFolderName = await fs.mkdtemp(`${name}-`);
  } catch (err: any) {
    await die(`Could not create \`${name}\`: ${err.message}`, tmpFolderName);
  }

  let pkg: Package | undefined;

  try {
    pkg = await Package.create(name, tmpFolderName);
    await fs.mkdir(path.join(tmpFolderName, "src"));
  } catch (err: any) {
    await die("Could not create `package.json`: " + err.message, tmpFolderName);
  }
  if (pkg) {
    for (let source of Object.keys(TEMPLATES)) {
      let target = path.join(tmpFolderName, TEMPLATES[source]);
      await expand(source, target, {
        package: pkg,
        versions: VERSIONS,
      });
    }
  }

  try {
    await fs.rename(tmpFolderName, name);
  } catch (err: any) {
    await die(`Could not create \`${name}\`: ${err.message}`, tmpFolderName);
  }
  console.log(`✨ Created Neon project \`${name}\`. Happy 🦀 hacking! ✨`);
}

if (process.argv.length < 3) {
  console.error(
    `✨ ${pink('create-neon:')} Create a new Neon project with zero configuration. ✨`
  );
  console.error();
  console.error(`${blue('Usage:')} npm init neon name`);
  console.error();
  console.error(
    "  name   The name of your Neon project, placed in a new directory of the same name."
  );
  console.error();
  process.exit(1);
}

main(process.argv[2]);

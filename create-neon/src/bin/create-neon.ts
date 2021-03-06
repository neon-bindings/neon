#!/usr/bin/env node

import { mkdir } from 'fs/promises';
import { Metadata } from '../metadata';
import * as path from 'path';
import die from '../die';
import Package from '../package';
import expand from '../expand';

const TEMPLATES: Record<string, string> = {
  '.gitignore.hbs': '.gitignore',
  'Cargo.toml.hbs': 'Cargo.toml',
  'README.md.hbs':  'README.md',
  'lib.rs.hbs':     path.join('src', 'lib.rs')
};

async function main(name: string) {
  try {
    await mkdir(name);
  } catch (err) {
    die(`Could not create \`${name}\`: ${err.message}`);
  }

  let pkg: Package;

  try {
    pkg = await Package.create(name);
  } catch (err) {
    die("Could not create `package.json`: " + err.message);
  }

  await mkdir(path.join(name, 'src'));

  let metadata = Metadata.from(pkg);

  for (let source of Object.keys(TEMPLATES)) {
    let target = path.join(name, TEMPLATES[source]);
    await expand(source, target, metadata);
  }

  console.log(`✨ Created Neon project \`${name}\`. Happy 🦀 hacking! ✨`);
}

if (process.argv.length !== 3) {
  console.error("✨ create-neon: Create a new Neon project with zero configuration. ✨");
  console.error();
  console.error("Usage: npm init neon name");
  console.error();
  console.error("  name   The name of your Neon project, placed in a new directory of the same name.");
  console.error();
  process.exit(1);
}

main(process.argv[2]);

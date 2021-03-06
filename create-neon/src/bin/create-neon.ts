#!/usr/bin/env node

import { mkdir } from 'fs/promises';
import * as path from 'path';
import die from '../die';
import Package from '../package';
import expand, { Versions } from '../expand';
import versions from '../../data/versions.json';

const TEMPLATES: Record<string, string> = {
  '.gitignore.hbs': '.gitignore',
  'Cargo.toml.hbs': 'Cargo.toml',
  'README.md.hbs':  'README.md',
  'lib.rs.hbs':     path.join('src', 'lib.rs')
};

function inferVersions(): Versions {
  // Select the N-API version associated with the current
  // running Node process.
  let inferred = process.versions.napi;

  let napi = inferred
    ? Math.min(Number(versions.napi), Number(inferred))
    : Number(versions.napi);

  return {
    neon: versions.neon,
    napi: napi
  };
}

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

  for (let source of Object.keys(TEMPLATES)) {
    let target = path.join(name, TEMPLATES[source]);
    await expand(source, target, {
      package: pkg,
      versions: inferVersions()
    });
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

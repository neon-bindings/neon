#!/usr/bin/env node

import { mkdir } from 'fs/promises';
import npmInit from '../npm-init';
import versions from '../../data/versions.json';
import { Project, Metadata } from '../metadata';
import * as path from 'path';
import Template from '../template';
import die from '../die';

async function main() {
  await npmInit();

  let project: Project;

  try {
    project = await Project.load('package.json');
  } catch (err) {
    die("Could not read `package.json`: " + err.message);
  }

  // Select the N-API version associated with the current
  // running Node process.
  let inferred = process.versions.napi;

  let napi = inferred
    ? Math.min(Number(versions.napi), Number(inferred))
    : Number(versions.napi);

  let metadata: Metadata = {
    project,
    versions: {
      neon: versions.neon,
      napi: napi
    }
  };

  await mkdir('src');

  let gitignore = new Template('.gitignore.hbs', '.gitignore');
  let manifest = new Template('Cargo.toml.hbs', 'Cargo.toml');
  let readme = new Template('README.md.hbs', 'README.md');
  let lib = new Template('lib.rs.hbs', path.join('src', 'lib.rs'));

  for (let template of [gitignore, manifest, readme, lib]) {
    try {
      await template.expand(metadata);
    } catch (err) {
      die(`Could not save ${template.target}: ${err.message}`);
    }
  }

  console.log(`✨ Initialized Neon project \`${metadata.project.name}\`. Happy 🦀 hacking! ✨`);
}

main();

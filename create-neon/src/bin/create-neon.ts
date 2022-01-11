#!/usr/bin/env node

import { promises as fs } from 'fs';
import * as path from 'path';
import die from '../die';
import Package from '../package';
import expand, { Versions } from '../expand';
import versions from '../../data/versions.json';
import os from 'os';

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
};

function deleteNeonDir(dir: string): Promise<void> {
  return fs.rm(dir, { force: true, recursive: true });
};

async function main(name: string) {
  let workingDirPath: string = process.cwd();
  let tmpDirPath: string = os.tmpdir();
  let tmpFolderName:string|undefined;

  try {
    //pretty lightweight way to check both that folder doesn't exist and
    // that the user has write permissions.
    await fs.mkdir(name);
    await fs.rmdir(name);

    // tmpFolderName = await fs.mkdtemp(path.join(os.tmpdir(), `${name}-`))
    tmpFolderName = await fs.mkdtemp(`${name}-`)

  } catch (err) {
    if (tmpFolderName) {
     await deleteNeonDir(tmpFolderName)
    };
    die(`Could not create \`${name}\`: ${err.message}`);
  };

  let pkg: Package;

  try {
      pkg = await Package.create(name,tmpFolderName);

  } catch (err) {
     await deleteNeonDir(tmpFolderName)
     die("Could not create `package.json`: " + err.message);
  };

  await fs.mkdir(path.join(tmpFolderName, 'src'));

  for (let source of Object.keys(TEMPLATES)) {
    let target = path.join(tmpFolderName, TEMPLATES[source]);
    await expand(source, target, {
      package: pkg,
      versions: inferVersions()
    });
  };
  try{
    // setting working directory back from tmpDirPath to workingDirPath shouldn't be required
    await fs.rename(tmpFolderName, path.join(workingDirPath,name))
  }
  catch(err){
    await deleteNeonDir(path.join(tmpDirPath,tmpFolderName))
    die(`Could not create \`${name}\`: ${err.message}`);
  };
  console.log(`✨ Created Neon project \`${name}\`. Happy 🦀 hacking! ✨`);
};

if (process.argv.length < 3) {
  console.error("✨ create-neon: Create a new Neon project with zero configuration. ✨");
  console.error();
  console.error("Usage: npm init neon name");
  console.error();
  console.error("  name   The name of your Neon project, placed in a new directory of the same name.");
  console.error();
  process.exit(1);
};

main(process.argv[2]);

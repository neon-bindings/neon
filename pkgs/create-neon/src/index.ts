import { promises as fs } from 'fs';
import * as path from 'path';
import die from './die.js';
import Package from './package.js';
import { VERSIONS } from './versions.js';
import expand from './expand.js';
import { Cache } from './cache.js';
import { CI } from './ci.js';

export type CreateNeonOptions = {
  templates: Record<string, string>,
  library?: boolean,
  cache?: Cache,
  ci?: CI,
  platforms?: string | string[]
};

export async function createNeon(name: string, options: CreateNeonOptions) {
  options.library ??= false;
  options.platforms ??= 'common';

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
    for (let source of Object.keys(options.templates)) {
      let target = path.join(tmpFolderName, options.templates[source]);
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

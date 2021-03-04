import { readFile, mkdir, writeFile } from 'fs/promises';
import npmInit from '../npm-init';
import versions from '../../data/versions.json';
import template from '../template';
import * as path from 'path';

function escapeQuotes(str: string): string {
  return str.replace(/"/g, '\\"');
}

async function main() {
  await npmInit();

  let name: string;
  let version: string;
  let author: string;
  let license: string;
  let description: string;

  try {
    let json = JSON.parse(await readFile('package.json', 'utf8'));
    name = json.name || "";
    version = json.version || "";
    author = escapeQuotes(json.author || "");
    license = json.license || "";
    description = escapeQuotes(json.description || "");
  } catch (err) {
    console.error("Could not read `package.json`: " + err.message);
    process.exit(1);
  }

  let ctx = {
    project: { name, version, author, license, description },
    versions
  };

  let gitignore = (await template('.gitignore.hbs'))(ctx);
  let manifest = (await template('Cargo.toml.hbs'))(ctx);
  let readme = (await template('README.md.hbs'))(ctx);
  let librs = (await template('lib.rs.hbs'))(ctx);

  await mkdir('src');

  await writeFile('.gitignore', gitignore, { flag: 'wx' });
  await writeFile('Cargo.toml', manifest, { flag: 'wx' });
  await writeFile('README.md', readme, { flag: 'wx' });
  await writeFile(path.join('src', 'lib.rs'), librs, { flag: 'wx' });

  console.log(`✨ Created Neon project \`${ctx.project.name}\`. Happy 🦀 hacking! ✨`);
}

main();

import { readFile, writeFile } from 'fs/promises';
import handlebars from 'handlebars';
import * as path from 'path';
import { Metadata } from './metadata';

const TEMPLATES_DIR = path.join(__dirname, '..', 'data', 'templates');

export default async function expand(source: string, target: string, metadata: Metadata) {
  let template = await readFile(path.join(TEMPLATES_DIR, source), 'utf8');
  let compiled = handlebars.compile(template, { noEscape: true });
  let expanded = compiled(metadata);
  // The 'wx' flag creates the file but fails if it already exists.
  await writeFile(target, expanded, { flag: 'wx' });
}

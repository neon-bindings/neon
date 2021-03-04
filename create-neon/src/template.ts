import { readFile } from 'fs/promises';
import handlebars from 'handlebars';
import * as path from 'path';

const TEMPLATES_DIR = path.join(__dirname, '..', 'data', 'templates');

export default async function template(filename: string) {
  let source = await readFile(path.join(TEMPLATES_DIR, filename), {
    encoding: 'utf8'
  });
  return handlebars.compile(source, { noEscape: true });
}

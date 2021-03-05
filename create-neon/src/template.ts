import { readFile, writeFile } from 'fs/promises';
import handlebars, { TemplateDelegate } from 'handlebars';
import * as path from 'path';
import { Metadata } from './metadata';

const TEMPLATES_DIR = path.join(__dirname, '..', 'data', 'templates');

export default class Template {
  source: string;
  target: string;
  private compiled: Promise<TemplateDelegate<Metadata>>;

  constructor(source: string, target: string) {
    this.source = source;
    this.target = target;
    this.compiled = readFile(path.join(TEMPLATES_DIR, source), {
      encoding: 'utf8'
    }).then(source => handlebars.compile(source, { noEscape: true }));
  }

  async expand(ctx: Metadata): Promise<null> {
    let expanded = (await this.compiled)(ctx);
    // The 'wx' flag creates the file but fails if it already exists.
    await writeFile(this.target, expanded, { flag: 'wx' });
    return null;
  }
}

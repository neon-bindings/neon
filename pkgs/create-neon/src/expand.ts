import { promises as fs } from "fs";
import handlebars from "handlebars";
import * as path from "path";
import Package from "./package.js";
import { Versions } from './versions.js';

const TEMPLATES_DIR = new URL(path.join('..', 'data', 'templates', '/'), import.meta.url);

export interface Metadata {
  package: Package;
  versions: Versions;
}

export default async function expand(
  source: string,
  target: string,
  metadata: Metadata
) {
  let template = await fs.readFile(new URL(source, TEMPLATES_DIR), "utf8");
  let compiled = handlebars.compile(template, { noEscape: true });
  let expanded = compiled(metadata);
  // The 'wx' flag creates the file but fails if it already exists.
  await fs.writeFile(target, expanded, { flag: "wx" });
}

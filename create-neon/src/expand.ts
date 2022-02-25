import { promises as fs } from "fs";
import handlebars from "handlebars";
import * as path from "path";
import Package from "./package";

const TEMPLATES_DIR = path.join(__dirname, "..", "data", "templates");

export interface Versions {
  neon: string;
  napi: number;
}

export interface Metadata {
  package: Package;
  versions: Versions;
}

export default async function expand(
  source: string,
  target: string,
  metadata: Metadata
) {
  let template = await fs.readFile(path.join(TEMPLATES_DIR, source), "utf8");
  let compiled = handlebars.compile(template, { noEscape: true });
  let expanded = compiled(metadata);
  // The 'wx' flag creates the file but fails if it already exists.
  await fs.writeFile(target, expanded, { flag: "wx" });
}

import { promises as fs } from "fs";
import * as path from "path";
import { npmInit } from "./shell.js";
import { VERSIONS } from "./versions.js";
import { Cache } from "./cache.js";
import { CI } from "./ci.js";
import { Metadata, expand, expandTo } from "./expand.js";
import { NodePlatform, PlatformPreset } from "@neon-rs/manifest/platform";

export enum Lang {
  JS = "js",
  DTS = "dts",
  TS = "ts",
}

export const LANG_TEMPLATES: Record<Lang, Record<string, string>> = {
  [Lang.JS]: {},
  [Lang.DTS]: {},
  [Lang.TS]: {
    "tsconfig.json.hbs": "tsconfig.json",
    "ts/index.cts.hbs": path.join("src", "index.cts"),
    "ts/index.mts.hbs": path.join("src", "index.mts"),
    "ts/load.cts.hbs": path.join("src", "load.cts"),
  },
};

export enum ModuleType {
  ESM = "esm",
  CJS = "cjs",
}

export type LibraryOptions = {
  lang: Lang;
  module: ModuleType;
  cache?: Cache;
  ci?: CI | undefined;
  platforms?: NodePlatform | PlatformPreset | (NodePlatform | PlatformPreset)[];
};

export type PackageOptions = {
  name: string;
  version: string;
  library: LibraryOptions | null;
  app: boolean | null;
  cache?: Cache | undefined;
  ci?: CI | undefined;
  interactive: boolean;
};

const KEYS = [
  "name",
  "version",
  "description",
  "main",
  "scripts",
  "author",
  "license",
];

function sort(json: any): any {
  // First copy the keys in the order specified in KEYS.
  let next = KEYS.filter((key) => json.hasOwnProperty(key))
    .map((key) => [key, json[key]])
    .reduce((acc, [key, val]) => Object.assign(acc, { [key]: val }), {});

  // Then copy any remaining keys in the original order.
  return Object.assign(next, json);
}

export default class Package {
  name: string;
  version: string;
  author: string;
  quotedAuthor: string;
  license: string;
  description: string;
  quotedDescription: string;

  static async create(
    metadata: Metadata,
    tmp: string,
    dir: string
  ): Promise<Package> {
    const baseTemplate = metadata.options.library
      ? "manifest/base/library.json.hbs"
      : "manifest/base/default.json.hbs";

    // 1. Load the base contents of the manifest from the base template.
    const seed = JSON.parse(await expand(baseTemplate, metadata));

    // 2. Mixin the scripts from the scripts template.
    seed.scripts = JSON.parse(
      await expand("manifest/scripts.json.hbs", metadata)
    );

    // 3. Mixin any scripts from the CI scripts template.
    if (metadata.options.library && metadata.options.library.ci) {
      const mixinTemplate = `ci/${metadata.options.library.ci.type}/manifest/scripts.json.hbs`;
      Object.assign(
        seed.scripts,
        JSON.parse(await expand(mixinTemplate, metadata))
      );
    }

    const filename = path.join(dir, "package.json");

    // 1. Write initial values to prevent `npm init` from asking unnecessary questions.
    await fs.writeFile(filename, JSON.stringify(seed));

    // 2. Call `npm init` to ask the user remaining questions.
    await npmInit(
      metadata.options.interactive,
      metadata.options.interactive ? [] : ["--yes"],
      dir,
      tmp
    );

    // 3. Sort the values in idiomatic `npm init` order.
    const sorted = sort(JSON.parse(await fs.readFile(filename, "utf8")));

    // 4. Save the result to package.json.
    await fs.writeFile(filename, JSON.stringify(sorted, undefined, 2) + "\n");

    return new Package(sorted);
  }

  constructor(json: any) {
    this.name = json.name;
    this.version = json.version;
    this.author = json.author;
    this.quotedAuthor = JSON.stringify(json.author);
    this.license = json.license;
    this.description = json.description;
    this.quotedDescription = JSON.stringify(json.description);
  }
}

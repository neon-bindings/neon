import die from "../die.js";
import { assertCanMkdir, mktemp } from "../fs.js";
import * as path from "node:path";
import * as fs from "node:fs/promises";
import { Context } from "../expand/context.js";
import { expand, expandTo } from "../expand/index.js";
import { npmInit } from "../shell.js";
import { Cache } from "../cache.js";
import { CI } from "../ci.js";
import { NodePlatform, PlatformPreset } from "@neon-rs/manifest/platform";

export enum Lang {
  JS = "js",
  DTS = "dts",
  TS = "ts",
}

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

export type ProjectOptions = {
  name: string;
  version: string;
  library: LibraryOptions | null;
  app: boolean | null;
  cache?: Cache | undefined;
  ci?: CI | undefined;
  interactive: boolean;
};

function stripNpmNamespace(pkg: string): string {
  return /^@[^/]+\/(?<stripped>.*)/.exec(pkg)?.groups?.stripped || pkg;
}

export abstract class Creator {
  protected _options: ProjectOptions;
  protected _temp: string = "";
  protected _tempPkg: string = "";

  static async for(options: ProjectOptions): Promise<Creator> {
    if (options.library) {
      const LibCreator = (await import("./lib.js")).LibCreator;
      return new LibCreator(options);
    } else {
      const AppCreator = (await import("./app.js")).AppCreator;
      return new AppCreator(options);
    }
  }

  constructor(options: ProjectOptions) {
    this._options = options;
  }

  async create(cx: Context): Promise<void> {
    try {
      await assertCanMkdir(this._options.name);

      this._temp = await mktemp();
      this._tempPkg = path.join(this._temp, this._options.name);

      await fs.mkdir(this._tempPkg);
    } catch (err: any) {
      await die(
        `Could not create \`${this._options.name}\`: ${err.message}`,
        this._temp
      );
    }

    await this.prepare(cx);

    const manifest = await npmInit(
      cx.options.interactive,
      cx.options.interactive ? [] : ["--yes"],
      this._tempPkg,
      this._temp
    );

    try {
      cx.package = {
        name: manifest.name,
        version: manifest.version,
        author: manifest.author,
        license: manifest.license,
        description: manifest.description,
      };

      const crateName = stripNpmNamespace(manifest.name);

      cx.crate = {
        name: crateName,
        version: manifest.version,
        author: manifest.author,
        description: manifest.description,
        license: manifest.license,
      };

      cx.crateStrings = {
        name: JSON.stringify(crateName),
        version: JSON.stringify(manifest.version),
        author: manifest.author ? JSON.stringify(manifest.author) : undefined,
        description: manifest.description
          ? JSON.stringify(manifest.description)
          : undefined,
        license: manifest.license
          ? JSON.stringify(manifest.license)
          : undefined,
      };
    } catch (err: any) {
      await die(
        "Could not create `package.json`: " + err.message,
        this._tempPkg
      );
    }

    await this.createNeonBoilerplate(cx);

    try {
      await fs.rename(this._tempPkg, this._options.name);
      await fs.rmdir(this._temp);
    } catch (err: any) {
      await die(
        `Could not create \`${this._options.name}\`: ${err.message}`,
        this._tempPkg
      );
    }
  }

  async createNeonBoilerplate(cx: Context): Promise<void> {
    const templates = this.templates(cx.package!.name);
    for (const source of Object.keys(templates)) {
      const target = path.join(this._tempPkg, templates[source]);
      await expandTo(source, target, cx);
    }
  }

  // Write initial values to prevent `npm init` from asking unnecessary questions.
  async prepare(cx: Context): Promise<void> {
    const template = `manifest/base/${this.baseTemplate()}`;

    const base = JSON.parse(await expand(template, cx));
    base.scripts = this.scripts();
    const filename = path.join(this._tempPkg, "package.json");
    await fs.writeFile(filename, JSON.stringify(base));
  }

  templates(_pkg: string): Record<string, string> {
    return {
      ".gitignore.hbs": ".gitignore",
      "Cargo.toml.hbs": "Cargo.toml",
      "README.md.hbs": "README.md",
      "lib.rs.hbs": path.join("src", "lib.rs"),
    };
  }

  scripts(): Record<string, string> {
    return {};
  }

  baseTemplate(): string {
    return "default.json.hbs";
  }
}

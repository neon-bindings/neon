import { Creator, ProjectOptions, LibraryOptions, Lang } from "./creator.js";
import { Context } from "../expand/context.js";
import * as path from "node:path";
import { expandTo } from "../expand/index.js";
import { LibraryManifest } from "@neon-rs/manifest";
import {
  NodePlatform,
  PlatformPreset,
  isNodePlatform,
} from "@neon-rs/manifest/platform";

const TS_TEMPLATES: Record<string, string> = {
  "tsconfig.json.hbs": "tsconfig.json",
  "ts/index.cts.hbs": path.join("src", "index.cts"),
  "ts/index.mts.hbs": path.join("src", "index.mts"),
  "ts/load.cts.hbs": path.join("src", "load.cts"),
};

export class LibCreator extends Creator {
  private _libOptions: LibraryOptions;

  constructor(options: ProjectOptions) {
    super(options);
    this._libOptions = options.library!;
    if (this._libOptions.ci) {
      this._libOptions.ci.setup();
    }
  }

  templates(pkg: string): Record<string, string> {
    return this._libOptions.lang === Lang.TS
      ? {
          ".gitignore.hbs": ".gitignore",
          "Cargo.toml.hbs": path.join("crates", pkg, "Cargo.toml"),
          "Workspace.toml.hbs": "Cargo.toml",
          "README.md.hbs": "README.md",
          "lib.rs.hbs": path.join("crates", pkg, "src", "lib.rs"),
        }
      : super.templates(pkg);
  }

  async createNeonBoilerplate(cx: Context): Promise<void> {
    await super.createNeonBoilerplate(cx);

    if (this._libOptions.lang === Lang.TS) {
      await this.createTSBoilerplate(cx);
    }

    if (this._libOptions.ci) {
      await this.createCIBoilerplate(cx);
    }

    await this.addPlatforms(cx);
  }

  async createTSBoilerplate(cx: Context): Promise<void> {
    for (const source of Object.keys(TS_TEMPLATES)) {
      const target = path.join(this._tempPkg, TS_TEMPLATES[source]);
      await expandTo(source, target, cx);
    }
  }

  async createCIBoilerplate(cx: Context): Promise<void> {
    const templates = this._libOptions.ci!.templates();
    for (const source of Object.keys(templates)) {
      const target = path.join(this._tempPkg, templates[source]);
      await expandTo(`ci/${this._libOptions.ci!.type}/${source}`, target, cx);
    }
  }

  async addPlatforms(cx: Context): Promise<void> {
    const manifest = await LibraryManifest.load(this._tempPkg);

    const platforms: (NodePlatform | PlatformPreset)[] = Array.isArray(
      this._libOptions.platforms
    )
      ? this._libOptions.platforms
      : !this._libOptions.platforms
      ? ["common"]
      : [this._libOptions.platforms];

    for (const platform of platforms) {
      if (isNodePlatform(platform)) {
        await manifest.addNodePlatform(platform);
      } else {
        await manifest.addPlatformPreset(platform);
      }
    }

    await manifest.saveChanges((msg) => {});
  }

  scripts(): Record<string, string> {
    const tscAnd = this._libOptions.lang === Lang.TS ? "tsc &&" : "";

    let scripts: Record<string, string> = {
      test: `${tscAnd}cargo test`,
      "cargo-build": `${tscAnd}cargo build --message-format=json > cargo.log`,
      "cross-build": `${tscAnd}cross build --message-format=json > cross.log`,
      "postcargo-build": "neon dist < cargo.log",
      "postcross-build": "neon dist -m /target < cross.log",
      debug: "npm run cargo-build --",
      build: "npm run cargo-build -- --release",
      cross: "npm run cross-build -- --release",
      prepack: `${tscAnd}neon update`,
      version: "neon bump --binaries platforms && git add .",
    };

    if (this._libOptions.ci) {
      Object.assign(scripts, this._libOptions.ci.scripts());
    }

    return scripts;
  }

  baseTemplate(): string {
    return "library.json.hbs";
  }
}

import { promises as fs } from "fs";
import * as path from "path";
import die from "./die.js";
import Package, {
  PackageOptions,
  Lang,
  ModuleType,
  LANG_TEMPLATES,
} from "./package.js";
import { Context } from "./expand/context.js";
import { expandTo } from "./expand/index.js";
import { LibraryManifest } from "@neon-rs/manifest";
import {
  NodePlatform,
  PlatformPreset,
  isNodePlatform,
  isPlatformPreset,
} from "@neon-rs/manifest/platform";
import { assertCanMkdir, mktemp } from "./fs.js";
import { Dialog, oneOf } from "./shell.js";
import { NPM } from "./cache/npm.js";
import { GitHub } from "./ci/github.js";

const CREATE_NEON_PRELUDE: string = `
This utility will walk you through creating a Neon project.
It only covers the most common items, and tries to guess sensible defaults.
  
Use \`npm install <pkg>\` afterwards to install a package and
save it as a dependency in the package.json file.
  
Use \`npm run build\` to build the Neon project from source.
  
Press ^C at any time to quit.      
`.trim();

async function askProjectType(options: PackageOptions) {
  const dialog = new Dialog();
  const ty = await dialog.ask({
    prompt: "project type",
    parse: oneOf({ app: "app" as const, lib: "lib" as const }),
    default: "app" as const,
    error: 'type should be a valid Neon project type ("app" or "lib").',
  });

  if (ty === "lib") {
    const platforms: (NodePlatform | PlatformPreset)[] = await dialog.ask({
      prompt: "target platforms",
      parse: (v: string): (NodePlatform | PlatformPreset)[] => {
        const a = v.split(",").map((s) => s.trim());
        if (a.some((elt) => !isNodePlatform(elt) && !isPlatformPreset(elt))) {
          throw new Error("parse error");
        }
        return a as (NodePlatform | PlatformPreset)[];
      },
      default: ["common"],
      error:
        "platforms should be a comma-separated list of platforms or platform presets.",
    });

    const cache = await dialog.ask({
      prompt: "binary cache",
      parse: oneOf({ npm: "npm" as const, none: undefined }),
      default: "npm" as const,
      error:
        'cache should be a supported Neon binary cache type ("npm" or "none").',
    });

    const org =
      cache === "npm"
        ? await dialog.ask({
            prompt: "cache org",
            parse: (v: string): string => v,
            default: NPM.inferOrg(options.name),
          })
        : null;

    const ci = await dialog.ask({
      prompt: "ci provider",
      parse: oneOf({ npm: "github" as const, none: undefined }),
      default: "github" as const,
      error:
        'provider should be a supported Neon CI provider ("github" or "none").',
    });

    options.library = {
      lang: Lang.TS,
      module: ModuleType.ESM,
      cache: cache === "npm" ? new NPM(options.name, org!) : undefined,
      ci: ci === "github" ? new GitHub() : undefined,
      platforms: platforms.length === 1 ? platforms[0] : platforms,
    };
  } else {
    options.app = true;
  }
  dialog.end();
}

export async function createNeon(templates: Record<string, string>, options: PackageOptions) {
  const cx = new Context(options);

  let tmpFolderName: string = "";
  let tmpPackagePath: string = "";

  try {
    await assertCanMkdir(options.name);

    tmpFolderName = await mktemp();
    tmpPackagePath = path.join(tmpFolderName, options.name);

    await fs.mkdir(tmpPackagePath);
  } catch (err: any) {
    await die(`Could not create \`${options.name}\`: ${err.message}`, tmpFolderName);
  }

  // Print a Neon variation of the `npm init` prelude text.
  if (options.interactive) {
    console.log(CREATE_NEON_PRELUDE);
  }

  // If neither --lib nor --app was specified, find out.
  if (options.library === null && options.app === null) {
    if (options.interactive) {
      await askProjectType(options);
    } else {
      options.app = true;
    }
  }

  try {
    cx.package = await Package.create(
      cx,
      tmpFolderName,
      tmpPackagePath
    );
  } catch (err: any) {
    await die(
      "Could not create `package.json`: " + err.message,
      tmpPackagePath
    );
  }

  if (options.library && options.library.ci) {
    options.library.ci.setup();
  }

  for (const source of Object.keys(templates)) {
    const target = path.join(tmpPackagePath, templates[source]);
    await expandTo(source, target, cx);
  }

  if (options.library) {
    const templates = LANG_TEMPLATES[options.library.lang];
    for (const source of Object.keys(templates)) {
      const target = path.join(tmpPackagePath, templates[source]);
      await expandTo(source, target, cx);
    }

    if (options.library.ci) {
      const templates = options.library.ci.templates();
      for (const source of Object.keys(templates)) {
        const target = path.join(tmpPackagePath, templates[source]);
        await expandTo(
          `ci/${options.library.ci.type}/${source}`,
          target,
          cx
        );
      }
    }

    const manifest = await LibraryManifest.load(tmpPackagePath);

    const platforms: (NodePlatform | PlatformPreset)[] = Array.isArray(
      options.library.platforms
    )
      ? options.library.platforms
      : !options.library.platforms
      ? ["common"]
      : [options.library.platforms];

    for (const platform of platforms) {
      if (isNodePlatform(platform)) {
        await manifest.addNodePlatform(platform);
      } else {
        await manifest.addPlatformPreset(platform);
      }
    }

    await manifest.saveChanges((msg) => {});
  }

  try {
    await fs.rename(tmpPackagePath, options.name);
    await fs.rmdir(tmpFolderName);
  } catch (err: any) {
    await die(`Could not create \`${options.name}\`: ${err.message}`, tmpFolderName);
  }

  console.log(`✨ Created Neon project \`${options.name}\`. Happy 🦀 hacking! ✨`);
}

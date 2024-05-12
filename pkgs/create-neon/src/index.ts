import { promises as fs } from 'fs';
import * as path from 'path';
import die from './die.js';
import Package, {
  PackageSpec,
  LibrarySpec,
  Lang,
  ModuleType,
  LANG_TEMPLATES,
} from "./package.js";
import { VERSIONS } from "./versions.js";
import { Metadata, expandTo } from "./expand.js";
import { LibraryManifest } from "@neon-rs/manifest";
import { NodePlatform, PlatformPreset, isNodePlatform, isPlatformPreset } from "@neon-rs/manifest/platform";
import { assertCanMkdir, mktemp } from './fs.js';
import { Dialog, oneOf } from './shell.js';
import { NPM } from './cache/npm.js';
import { GitHub } from './ci/github.js';

const CREATE_NEON_PRELUDE: string = `
This utility will walk you through creating a Neon project.
It only covers the most common items, and tries to guess sensible defaults.
  
Use \`npm install <pkg>\` afterwards to install a package and
save it as a dependency in the package.json file.
  
Use \`npm run build\` to build the Neon project from source.
  
Press ^C at any time to quit.      
`.trim();

async function askProjectType(packageSpec: PackageSpec) {
  // If non-interactive, use the default (--app).
  if (packageSpec.yes) {
    packageSpec.app = true;
    return;
  }

  // Otherwise, find out interactively.
  const dialog = new Dialog();
  const ty = await dialog.ask({
    prompt: 'project type',
    parse: oneOf({ app: 'app' as const, lib: 'lib' as const }),
    default: 'app' as const,
    error: "type should be a valid Neon project type (\"app\" or \"lib\")."
  });

  if (ty === 'lib') {
    const platforms: (NodePlatform | PlatformPreset)[] = await dialog.ask({
      prompt: 'target platforms',
      parse: (v: string): (NodePlatform | PlatformPreset)[] => {
        const a = v.split(',').map(s => s.trim());
        if (a.some(elt => !isNodePlatform(elt) && !isPlatformPreset(elt))) {
          throw new Error("parse error");
        }
        return a as (NodePlatform | PlatformPreset)[];
      },
      default: ['common'],
      error: "platforms should be a comma-separated list of platforms or platform presets."
    });

    const cache = await dialog.ask({
      prompt: 'binary cache',
      parse: oneOf({ npm: 'npm' as const, none: undefined }),
      default: 'npm' as const,
      error: "cache should be a supported Neon binary cache type (\"npm\" or \"none\")."
    });

    const org = cache === 'npm' ? await dialog.ask({
      prompt: 'cache org',
      parse: (v: string): string => v,
      default: NPM.inferOrg(packageSpec.name)
    }) : null;

    const ci = await dialog.ask({
      prompt: 'ci provider',
      parse: oneOf({ npm: 'github' as const, none: undefined }),
      default: 'github' as const,
      error: "provider should be a supported Neon CI provider (\"github\" or \"none\")."
    });

    packageSpec.library = {
      lang: Lang.TS,
      module: ModuleType.ESM,
      cache: cache === 'npm' ? new NPM(packageSpec.name, org!) : undefined,
      ci: ci === 'github' ? new GitHub() : undefined,
      platforms: (platforms.length === 1) ? platforms[0] : platforms
    };
  } else {
    packageSpec.app = true;
  }
  dialog.end();
}

export type CreateNeonOptions = {
  templates: Record<string, string>;
  library: LibrarySpec | null;
  app: boolean | null;
};

export async function createNeon(name: string, options: CreateNeonOptions) {
  const packageSpec: PackageSpec = {
    name,
    version: "0.1.0",
    library: options.library,
    app: options.app,
    // Even if the user specifies this with a flag (e.g. `npm init -y neon`),
    // `npm init` sets this env var to 'true' before invoking create-neon.
    // So this is the most general way to check this configuration option.
    yes: process.env['npm_configure_yes'] === 'true',
  };

  const metadata: Metadata = {
    packageSpec,
    versions: VERSIONS,
  };

  let tmpFolderName: string = "";
  let tmpPackagePath: string = "";

  try {
    await assertCanMkdir(name);

    tmpFolderName = await mktemp();
    tmpPackagePath = path.join(tmpFolderName, name);

    await fs.mkdir(tmpPackagePath);
  } catch (err: any) {
    await die(`Could not create \`${name}\`: ${err.message}`, tmpFolderName);
  }

  // Print a Neon variation of the `npm init` prelude text.
  if (!packageSpec.yes) {
    console.log(CREATE_NEON_PRELUDE);
  }

  // If neither --lib nor --app was specified, find out.
  if (packageSpec.library === null && packageSpec.app === null) {
    await askProjectType(packageSpec);
  }

  try {
    metadata.package = await Package.create(metadata, tmpFolderName, tmpPackagePath);
  } catch (err: any) {
    await die(
      "Could not create `package.json`: " + err.message,
      tmpPackagePath
    );
  }

  if (metadata.package) {
    if (packageSpec.library && packageSpec.library.ci) {
      packageSpec.library.ci.setup();
    }

    for (const source of Object.keys(options.templates)) {
      const target = path.join(tmpPackagePath, options.templates[source]);
      await expandTo(source, target, metadata);
    }
  }

  if (packageSpec.library) {
    const templates = LANG_TEMPLATES[packageSpec.library.lang];
    for (const source of Object.keys(templates)) {
      const target = path.join(tmpPackagePath, templates[source]);
      await expandTo(source, target, metadata);
    }

    if (packageSpec.library.ci) {
      const templates = packageSpec.library.ci.templates();
      for (const source of Object.keys(templates)) {
        const target = path.join(tmpPackagePath, templates[source]);
        await expandTo(
          `ci/${packageSpec.library.ci.type}/${source}`,
          target,
          metadata
        );
      }
    }

    const manifest = await LibraryManifest.load(tmpPackagePath);

    const platforms: (NodePlatform | PlatformPreset)[] = Array.isArray(
      packageSpec.library.platforms
    )
      ? packageSpec.library.platforms
      : !packageSpec.library.platforms
      ? ['common']
      : [packageSpec.library.platforms];

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
    await fs.rename(tmpPackagePath, name);
    await fs.rmdir(tmpFolderName);
  } catch (err: any) {
    await die(`Could not create \`${name}\`: ${err.message}`, tmpFolderName);
  }

  console.log(`✨ Created Neon project \`${name}\`. Happy 🦀 hacking! ✨`);
}

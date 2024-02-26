import { promises as fs } from 'fs';
import * as path from 'path';
import die from './die.js';
import Package, { PackageSpec, LibrarySpec, Lang, ModuleType, LANG_TEMPLATES } from './package.js';
import { VERSIONS } from './versions.js';
import { Metadata, expandTo } from './expand.js';
import { LibraryManifest } from '@neon-rs/manifest';
import { PlatformPreset } from '@neon-rs/manifest/platform';

export type CreateNeonOptions = {
  templates: Record<string, string>,
  library: LibrarySpec | null,
  yes: boolean | undefined,
};

export async function createNeon(name: string, options: CreateNeonOptions) {
  const packageSpec: PackageSpec = {
    name,
    library: options.library,
    yes: options.yes
  };

  const metadata: Metadata = {
    packageSpec,
    versions: VERSIONS
  };

  let tmpFolderName: string = "";
  let tmpPackagePath: string = "";

  try {
    // pretty lightweight way to check both that folder doesn't exist and
    // that the user has write permissions.
    await fs.mkdir(name);
    await fs.rmdir(name);

    tmpFolderName = await fs.mkdtemp(`neon-`);
    tmpPackagePath = path.join(tmpFolderName, name);
    await fs.mkdir(tmpPackagePath);
  } catch (err: any) {
    await die(`Could not create \`${name}\`: ${err.message}`, tmpFolderName);
  }

  let pkg: Package | undefined;

  try {
    pkg = await Package.create(metadata, tmpPackagePath);
    metadata.package = pkg;
    await fs.mkdir(path.join(tmpPackagePath, "src"));
  } catch (err: any) {
    await die("Could not create `package.json`: " + err.message, tmpPackagePath);
  }
  if (pkg) {
    for (const source of Object.keys(options.templates)) {
      const target = path.join(tmpPackagePath, options.templates[source]);
      await expandTo(source, target, metadata);
    }
  }

  if (options.library) {
    const templates = LANG_TEMPLATES[options.library.lang];
    for (const source of Object.keys(templates)) {
      const target = path.join(tmpPackagePath, templates[source]);
      await expandTo(source, target, metadata);
    }

    if (options.library.ci) {
      const templates = options.library.ci.templates();
      for (const source of Object.keys(templates)) {
        const target = path.join(tmpPackagePath, templates[source]);
        await expandTo(`ci/${options.library.ci.type}/${source}`, target, metadata);
      }
    }

    const manifest = await LibraryManifest.load(tmpPackagePath);

    const platformPresets: PlatformPreset[] = Array.isArray(options.library.platforms)
      ? options.library.platforms
      : !options.library.platforms
      ? ['common']
      : [options.library.platforms];

    for (const preset of platformPresets) {
      await manifest.addPlatformPreset(preset);
    }

    await manifest.saveChanges(msg => {});
  }

  try {
    await fs.rename(tmpPackagePath, name);
    await fs.rmdir(tmpFolderName);
  } catch (err: any) {
    await die(`Could not create \`${name}\`: ${err.message}`, tmpFolderName);
  }

  console.log(`✨ Created Neon project \`${name}\`. Happy 🦀 hacking! ✨`);
}

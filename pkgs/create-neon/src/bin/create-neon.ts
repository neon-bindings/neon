#!/usr/bin/env node

import * as path from 'path';
import commandLineArgs from 'command-line-args';
import { printErrorWithUsage } from '../print.js';
import { createNeon } from '../index.js';
import { Cache } from '../cache.js';
import { NPM } from '../cache/npm.js';
import { CI } from '../ci.js';
import { GitHub } from '../ci/github.js';
import { Lang, ModuleType } from '../package.js';
import { PlatformPreset, assertIsPlatformPreset, isPlatformPreset } from '@neon-rs/manifest/platform';

const TEMPLATES: Record<string, string> = {
  ".gitignore.hbs": ".gitignore",
  ".npmignore.hbs": ".npmignore",
  "Cargo.toml.hbs": "Cargo.toml",
  "README.md.hbs": "README.md",
  "lib.rs.hbs": path.join("src", "lib.rs"),
};

const OPTIONS = [
  { name: 'lib', type: Boolean, defaultValue: false },
  { name: 'bins', type: String, defaultValue: 'none' },
  { name: 'platform', type: String, multiple: true, defaultValue: ['common'] },
  { name: 'ci', alias: 'c', type: String, defaultValue: 'github' },
  { name: 'yes', alias: 'y', type: Boolean, defaultValue: false }
];

try {
  const opts = commandLineArgs(OPTIONS, { stopAtFirstUnknown: true });

  if (!opts._unknown || opts._unknown.length === 0) {
    throw new Error('No package name given');
  }

  if (opts._unknown.length > 1) {
    throw new Error(`unexpected argument (${opts._unknown[1]})`);
  }

  const [pkg] = opts._unknown;
  const platforms = parsePlatforms(opts.platform);
  const cache = parseCache(opts.lib, opts.bins, pkg);
  const ci = parseCI(opts.ci);
  const yes = !!opts.yes;

  createNeon(pkg, {
    templates: TEMPLATES,
    library: opts.lib ? {
      lang: Lang.TS,
      module: ModuleType.ESM,
      cache,
      ci,
      platforms
    } : null,
    yes
  });
} catch (e) {
  printErrorWithUsage(e);
  process.exit(1);
}

function parsePlatforms(platforms: string[]): PlatformPreset | PlatformPreset[] | undefined {
  if (platforms.length === 0) {
    return undefined;
  } else if (platforms.length === 1) {
    const preset = platforms[0];
    assertIsPlatformPreset(preset);
    return preset;
  } else {
    return platforms.map(preset => {
      assertIsPlatformPreset(preset);
      return preset;
    });
  }
}

function parseCI(ci: string): CI | undefined {
  switch (ci) {
    case 'none': return undefined;
    case 'github': return new GitHub();
    default:
      throw new Error(`Unrecognized CI system ${ci}, expected 'github' or 'none'`);
  }
}

function parseCache(lib: boolean, bins: string, pkg: string): Cache | undefined {
  const defaultOrg = '@' + pkg;

  if (bins === 'none') {
    return lib ? new NPM(defaultOrg) : undefined;
  }

  if (bins === 'npm') {
    return new NPM(defaultOrg);
  }

  if (bins.startsWith('npm:')) {
    return new NPM(bins.substring(4));
  }

  throw new Error(`Unrecognized binaries cache ${bins}, expected 'npm[:org]' or 'none'`)
}

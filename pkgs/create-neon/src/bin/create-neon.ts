#!/usr/bin/env node

import * as path from 'path';
import commandLineArgs from 'command-line-args';
import { printErrorWithUsage } from '../print.js';
import { createNeon } from '../index.js';
import { Cache } from '../cache.js';
import { NPM } from '../cache/npm.js';
import { CI } from '../ci.js';
import { GitHub } from '../ci/github.js';

const TEMPLATES: Record<string, string> = {
  ".gitignore.hbs": ".gitignore",
  "Cargo.toml.hbs": "Cargo.toml",
  "README.md.hbs": "README.md",
  "lib.rs.hbs": path.join("src", "lib.rs"),
};

const OPTIONS = [
  { name: 'lib', type: Boolean, defaultValue: false },
  { name: 'bins', type: String, defaultValue: 'none' },
  { name: 'platform', type: String, multiple: true, defaultValue: [] },
  { name: 'ci', alias: 'c', type: String, defaultValue: 'github' }
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

  createNeon(pkg, {
    templates: TEMPLATES,
    library: opts.lib,
    cache,
    ci,
    platforms
  });
} catch (e) {
  printErrorWithUsage(e);
  process.exit(1);
}

function parsePlatforms(platforms: string[]): string | string[] | undefined {
  if (platforms.length === 0) {
    return undefined;
  } else if (platforms.length === 1) {
    return platforms[0];
  } else {
    return platforms;
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

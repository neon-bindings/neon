#!/usr/bin/env node

import commandLineArgs from "command-line-args";
import { printErrorWithUsage } from "../print.js";
import { createNeon } from "../index.js";
import { Cache } from "../cache.js";
import { NPM } from "../cache/npm.js";
import { CI } from "../ci.js";
import { GitHub } from "../ci/github.js";
import { Lang, ModuleType } from "../create/creator.js";
import {
  NodePlatform,
  PlatformPreset,
  isNodePlatform,
  isPlatformPreset,
} from "@neon-rs/manifest/platform";

const OPTIONS = [
  { name: "app", type: Boolean, defaultValue: false },
  { name: "lib", type: Boolean, defaultValue: false },
  { name: "bins", type: String, defaultValue: "none" },
  { name: "platform", type: String, multiple: true, defaultValue: ["common"] },
  { name: "ci", alias: "c", type: String, defaultValue: "github" },
  { name: "yes", alias: "y", type: Boolean, defaultValue: false },
];

try {
  const opts = commandLineArgs(OPTIONS, { stopAtFirstUnknown: true });

  if (opts.app && opts.lib) {
    throw new Error("Cannot choose both --app and --lib");
  }

  if (!opts._unknown || opts._unknown.length === 0) {
    throw new Error("No package name given");
  }

  if (opts._unknown.length > 1) {
    throw new Error(`unexpected argument (${opts._unknown[1]})`);
  }

  const [pkg] = opts._unknown;
  const { org, basename } = /^((?<org>@[^/]+)\/)?(?<basename>.*)/.exec(pkg)
    ?.groups as {
    org?: string;
    basename: string;
  };
  const fullName = org ? pkg : basename;
  const platforms = parsePlatforms(opts.platform);
  const cache = parseCache(opts.lib, opts.bins, basename, org);
  const ci = parseCI(opts.ci);

  if (opts.yes) {
    process.env["npm_configure_yes"] = "true";
  }

  createNeon({
    org,
    basename,
    fullName,
    version: "0.1.0",
    library: opts.lib
      ? {
          lang: Lang.TS,
          module: ModuleType.ESM,
          cache,
          ci,
          platforms,
        }
      : null,
    app: opts.app ? true : opts.lib ? false : null,
    // Even if the user specifies this with a flag (e.g. `npm init -y neon`),
    // `npm init` sets this env var to 'true' before invoking create-neon.
    // So this is the most general way to check this configuration option.
    interactive: process.env["npm_configure_yes"] !== "true",
  });
} catch (e) {
  printErrorWithUsage(e);
  process.exit(1);
}

function parsePlatforms(
  platforms: string[]
):
  | NodePlatform
  | PlatformPreset
  | (NodePlatform | PlatformPreset)[]
  | undefined {
  if (platforms.length === 0) {
    return undefined;
  } else if (platforms.length === 1) {
    const platform = platforms[0];
    if (isNodePlatform(platform) || isPlatformPreset(platform)) {
      return platform;
    }
    throw new TypeError(`expected platform or preset, got ${platform}`);
  } else {
    return platforms.map((platform) => {
      if (isNodePlatform(platform) || isPlatformPreset(platform)) {
        return platform;
      }
      throw new TypeError(`expected platform or preset, got ${platform}`);
    });
  }
}

function parseCI(ci: string): CI | undefined {
  switch (ci) {
    case "none":
      return undefined;
    case "github":
      return new GitHub();
    default:
      throw new Error(
        `Unrecognized CI system ${ci}, expected 'github' or 'none'`
      );
  }
}

function parseCache(
  lib: boolean,
  bins: string,
  pkg: string,
  org: string | undefined
): Cache | undefined {
  const defaultPrefix = org ? `${pkg}-` : "";
  org ??= `@${pkg}`;

  // CASE: npm create neon -- --app logos-r-us
  // CASE: npm create neon -- --app @acme/logos-r-us
  //   - <no binaries cache>
  if (bins === "none" && !lib) {
    return undefined;
  }

  // CASE: npm create neon -- --lib logo-generator
  // CASE: npm create neon -- --lib --bins npm logo-generator
  //   - lib: `logo-generator`
  //   - bin: `@logo-generator/darwin-arm64`

  // CASE: npm create neon -- --lib @acme/logo-generator
  // CASE: npm create neon -- --lib --bins npm @acme/logo-generator
  //   - lib: `@acme/logo-generator`
  //   - bin: `@acme/logo-generator-darwin-arm64`
  if (bins === "none" || bins === "npm") {
    return new NPM(org, defaultPrefix);
  }

  // CASE: npm create neon -- --lib --bins=npm:acme logo-generator
  //   lib: logo-generator
  //   bin: @acme/logo-generator-darwin-arm64

  // CASE: npm create neon -- --lib --bins=npm:acme/libs-logo-generator- logo-generator
  //   lib: logo-generator
  //   bin: @acme/libs-logo-generator-darwin-arm64

  // CASE: npm create neon -- --lib --bins=npm:acme-libs @acme/logo-generator
  //   lib: @acme-libs/logo-generator
  //   bin: @acme-libs/logo-generator-darwin-arm64
  if (bins.startsWith("npm:")) {
    const split = bins.substring(4).split("/", 2);
    const org = split[0].replace(/^@?/, "@"); // don't care if they include the @ or not
    const prefix = split.length > 1 ? split[1] : defaultPrefix;
    return new NPM(org, prefix);
  }

  throw new Error(
    `Unrecognized binaries cache ${bins}, expected 'npm[:org[/prefix]]' or 'none'`
  );
}

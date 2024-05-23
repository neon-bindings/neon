#!/usr/bin/env node

import * as path from "path";
import commandLineArgs from "command-line-args";
import { printErrorWithUsage } from "../print.js";
import { createNeon } from "../index.js";
import { Cache } from "../cache.js";
import { NPM } from "../cache/npm.js";
import { CI } from "../ci.js";
import { GitHub } from "../ci/github.js";
import { Lang, ModuleType } from "../package.js";
import {
  NodePlatform,
  PlatformPreset,
  assertIsPlatformPreset,
  isNodePlatform,
  isPlatformPreset,
} from "@neon-rs/manifest/platform";

const JS_TEMPLATES: Record<string, string> = {
  ".gitignore.hbs": ".gitignore",
  "Cargo.toml.hbs": "Cargo.toml",
  "README.md.hbs": "README.md",
  "lib.rs.hbs": path.join("src", "lib.rs"),
};

function tsTemplates(pkg: string): Record<string, string> {
  return {
    ".gitignore.hbs": ".gitignore",
    "Cargo.toml.hbs": path.join("crates", pkg, "Cargo.toml"),
    "Workspace.toml.hbs": "Cargo.toml",
    "README.md.hbs": "README.md",
    "lib.rs.hbs": path.join("crates", pkg, "src", "lib.rs"),
  };
}

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
  const platforms = parsePlatforms(opts.platform);
  const cache = parseCache(opts.lib, opts.bins, pkg);
  const ci = parseCI(opts.ci);

  if (opts.yes) {
    process.env["npm_configure_yes"] = "true";
  }

  createNeon(opts.lib ? tsTemplates(pkg) : JS_TEMPLATES, {
    name: pkg,
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
  pkg: string
): Cache | undefined {
  if (bins === "none") {
    return lib ? new NPM(pkg) : undefined;
  }

  if (bins === "npm") {
    return new NPM(pkg);
  }

  if (bins.startsWith("npm:")) {
    return new NPM(pkg, bins.substring(4));
  }

  throw new Error(
    `Unrecognized binaries cache ${bins}, expected 'npm[:org]' or 'none'`
  );
}

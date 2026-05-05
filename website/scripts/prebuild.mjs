// Astro prebuild orchestrator: regenerate rustdoc and copy it into
// public/api/ unless SKIP_RUSTDOC is set. Use SKIP_RUSTDOC=1 npm run build
// for a fast Astro-only iteration; CI and Netlify always rebuild.
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const websiteDir = resolve(here, "..");
const repoRoot = resolve(websiteDir, "..");

if (process.env.SKIP_RUSTDOC) {
  const apiDir = resolve(websiteDir, "public", "api");
  if (!existsSync(apiDir)) {
    console.error(
      `[prebuild] SKIP_RUSTDOC is set but ${apiDir} does not exist; ` +
        `Astro will build without /api/. Run once without SKIP_RUSTDOC to ` +
        `populate it.`,
    );
  } else {
    console.log("[prebuild] SKIP_RUSTDOC set; skipping cargo doc.");
  }
  process.exit(0);
}

console.log("[prebuild] Running cargo doc -p neon --no-deps...");
const cargo = spawnSync(
  "cargo",
  [
    "doc",
    "-p",
    "neon",
    "--no-deps",
    "--manifest-path",
    resolve(repoRoot, "Cargo.toml"),
  ],
  { stdio: "inherit" },
);
if (cargo.status !== 0) {
  process.exit(cargo.status ?? 1);
}

console.log("[prebuild] Copying rustdoc HTML...");
const copy = spawnSync("node", [resolve(here, "copy-rustdoc.mjs")], {
  stdio: "inherit",
});
process.exit(copy.status ?? 1);

import { cp, rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..");

// Honor CARGO_TARGET_DIR so this works under sandboxed/CI environments
// that redirect cargo output (e.g. Cursor's local sandbox).
const targetDir = process.env.CARGO_TARGET_DIR
  ? resolve(process.env.CARGO_TARGET_DIR)
  : resolve(repoRoot, "target");
const src = resolve(targetDir, "doc");
const dst = resolve(here, "..", "public", "api");

console.log(`Copying rustdoc HTML from ${src} to ${dst}`);
await rm(dst, { recursive: true, force: true });
await cp(src, dst, { recursive: true });
console.log("Done.");

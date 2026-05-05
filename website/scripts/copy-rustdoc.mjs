import { cp, rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..");
const src = resolve(repoRoot, "target", "doc");
const dst = resolve(here, "..", "public", "api");

console.log(`Copying rustdoc HTML from ${src} to ${dst}`);
await rm(dst, { recursive: true, force: true });
await cp(src, dst, { recursive: true });
console.log("Done.");

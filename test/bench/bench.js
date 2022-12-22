"use strict";

const bench = require(".");

const args = process.argv.slice(2);
const benchArgs = args.filter((f) => !f.startsWith("-"));
const benches = new Set(benchArgs.length ? benchArgs : Object.keys(bench));

const flags = new Set(
  args
    .filter((f) => f.startsWith("-") && !f.startsWith("--report"))
    .map((f) => f.slice(2))
);

const [reportFile] = args
  .filter((f) => f.startsWith("--report="))
  .map((f) => f.slice(9));

const options = {
  neon: flags.size === 0 ? true : flags.has("neon"),
  json: flags.size === 0 ? true : flags.has("json"),
  reportFile,
};

for (const [name, b] of Object.entries(bench)) {
  if (benches.has(name)) {
    b(options);
  }
}

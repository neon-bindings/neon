"use strict";

const { spawn } = require("child_process");
const { promises: { copyFile, mkdir, stat } } = require("fs");
const { dirname } = require("path");
const readline = require("readline");

const { ParseError, getArtifactName, parse } = require("./args");

function run(argv, env) {
  const options = parseArgs(argv, env);
  const copied = {};

  const cp = spawn(options.cmd, options.args, {
    stdio: ["inherit", "pipe", "inherit"]
  });

  const rl = readline.createInterface({ input: cp.stdout });

  cp.on("error", (err) => {
    if (options.cmd === "cargo" && err.code === "ENOENT") {
      console.error(`Error: could not find the \`cargo\` executable.

You can find instructions for installing Rust and Cargo at:

    https://www.rust-lang.org/tools/install

`);
    } else {
      console.error(err);
    }
    process.exitCode = 1;
  });

  cp.on("exit", (code) => {
    if (!process.exitCode) {
      process.exitCode = code;
    }
  });

  rl.on("line", (line) => {
    try {
      processCargoBuildLine(options, copied, line);
    } catch (err) {
      console.error(err);
      process.exitCode = 1;
    }
  });

  process.on("exit", () => {
    Object.keys(options.artifacts).forEach((name) => {
      if (!copied[name]) {
        console.error(`Did not copy "${name}"`);

        if (!process.exitCode) {
          process.exitCode = 1;
        }
      }
    });
  });
}

function processCargoBuildLine(options, copied, line) {
  const data = JSON.parse(line);
  const { filenames, reason, target } = data;

  if (!data || reason !== "compiler-artifact" || !target) {
    return;
  }

  const { kind: kinds, name } = data.target;

  if (!Array.isArray(kinds) || !Array.isArray(filenames)) {
    return;
  }

  // `kind` and `filenames` zip up as key/value pairs
  kinds.forEach((kind, i) => {
    const filename = filenames[i];
    const key = getArtifactName({ artifactType: kind, crateName: name });
    const outputFiles = options.artifacts[key];

    if (!outputFiles || !filename) {
      return;
    }

    Promise
        .all(outputFiles.map(outputFile => copyArtifact(filename, outputFile)))
        .then(() => {
          copied[key] = true;
        })
        .catch((err) => {
          process.exitCode = 1;
          console.error(err);
        });
  });
}

async function isNewer(filename, outputFile) {
  try {
    const prevStats = await stat(outputFile);
    const nextStats = await stat(filename);

    return nextStats.mtime > prevStats.mtime;
  } catch (_err) {}

  return true;
}

async function copyArtifact(filename, outputFile) {
  if (!(await isNewer(filename, outputFile))) {
    return;
  }

  const outputDir = dirname(outputFile);

  // Don't try to create the current directory
  if (outputDir && outputDir !== ".") {
    await mkdir(outputDir, { recursive: true });
  }

  await copyFile(filename, outputFile);
}

function parseArgs(argv, env) {
  try {
    return parse(argv, env);
  } catch (err) {
    if (err instanceof ParseError) {
      quitError(err.message);
    } else {
      throw err;
    }
  }
}

function quitError(msg) {
  console.error(msg);
  process.exit(1);
}

module.exports = run;

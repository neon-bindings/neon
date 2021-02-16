#!/usr/bin/env node
"use strict";

const { spawn } = require("child_process");
const { promises: { copyFile, mkdir, stat } } = require("fs");
const { dirname } = require("path");
const readline = require("readline");

const options = parseArgs(process.argv.slice(2));
const copied = {};

const cp = spawn(options.command, options.arguments, {
  stdio: ["inherit", "pipe", "inherit"]
});

const rl = readline.createInterface({ input: cp.stdout });

cp.on("error", (err) => {
  console.error(err);
  process.exitCode = 1;
});

cp.on("exit", (code) => {
  if (!process.exitCode) {
    process.exitCode = code;
  }
});

rl.on("line", (line) => {
  try {
    processCargoBuildLine(line);
  } catch (err) {
    console.error(err);
    process.exitCode = 1;
  }
});

process.on("exit", () => {
  Object.keys(options.outputFiles).forEach((name) => {
    if (!copied[name]) {
      console.error(`Did not copy "${name}"`);

      if (!process.exitCode) {
        process.exitCode = 1;
      }
    }
  });
});

function processCargoBuildLine(line) {
  const data = JSON.parse(line);

  if (!data || data.reason !== "compiler-artifact" || !data.target) {
    return;
  }

  const { kind: kinds, name } = data.target;

  if (!Array.isArray(kinds) || !kinds.length) {
    return;
  }

  const [kind] = kinds;
  const key = makeKey(name, kind);
  const artifactConfig = options.outputFiles[key];

  if (!artifactConfig || !Array.isArray(data.filenames)) {
    return;
  }

  const [filename] = data.filenames;
  const { outputFile } = artifactConfig;

  if (!filename) {
    return;
  }

  copyArtifact(filename, outputFile)
    .then(() => {
      copied[key] = true;
    })
    .catch((err) => {
      process.exitCode = 1;
      console.error(err);
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

  await mkdir(dirname(outputFile), { recursive: true });
  await copyFile(filename, outputFile);
}

// Expects: Options and command separated by "--"
// Example: "arguments to CLI -- command to execute"
function parseArgs(args) {
  const splitAt = args.indexOf("--");
  const options = splitAt >= 0 ? args.slice(0, splitAt) : args;
  const command = splitAt >= 0 ? args.slice(splitAt + 1) : [];
  const outputFiles = parseOutputFiles(options);

  if (!command.length) {
    quitError([
      "Missing command to execute.",
      [
        "cargo-cp-artifct my-crate=cdylib=index.node",
        "--",
        "cargo build --message-format=json-render-diagnostics"
      ].join(" ")
    ].join("\n"));
  }

  return {
    command: command[0],
    arguments: command.slice(1),
    outputFiles
  };
}

// Expects: List of "crate-name=kind=output_file_path" sets
function parseOutputFiles(args) {
  return args
    .map(parseOutputFile)
    .reduce((acc, opts) => ({
      ...acc,
      [makeKey(opts.crateName, opts.kind)]: opts
    }), {});
}

// Expects: "crate-name=kind=output_file_path" set
function parseOutputFile(opts) {
  const nameSplitAt = opts.indexOf("=");

  if (nameSplitAt < 0) {
    quitError(`Missing artifact kind: ${opts}`);
  }

  const crateNameRaw = opts.slice(0, nameSplitAt);
  const crateName = crateNameRaw === "$npm_package_name"
    ? process.env.npm_package_name
    : crateNameRaw;

  const remainder = opts.slice(nameSplitAt + 1);
  const kindSplitAt = remainder.indexOf("=");

  if (kindSplitAt < 0) {
    quitError(`Missing output file name: ${opts}`);
  }

  const kind = remainder.slice(0, kindSplitAt);
  const outputFile = remainder.slice(kindSplitAt + 1);

  return { crateName, kind, outputFile };
}

function makeKey(crateName, kind) {
  return `${crateName}=${kind}`;
}

function quitError(msg) {
  console.error(msg);
  process.exit(1);
}

"use strict";

const { spawn } = require("child_process");
const {
  promises: { copyFile, mkdir, stat, unlink },
} = require("fs");
const { dirname, extname } = require("path");
const readline = require("readline");

const { ParseError, getArtifactName, parse } = require("./args");

function run(argv, env) {
  const options = parseArgs(argv, env);
  const copied = {};

  const cp = spawn(options.cmd, options.args, {
    stdio: ["inherit", "pipe", "inherit"],
    shell: process.platform === "win32",
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
    const { key, outputFiles } =
      getOutputFiles(kind, name, options.artifacts) || {};

    if (!outputFiles || !filename) {
      return;
    }

    Promise.all(
      outputFiles.map((outputFile) => copyArtifact(filename, outputFile))
    )
      .then(() => {
        copied[key] = true;
      })
      .catch((err) => {
        process.exitCode = 1;
        console.error(err);
      });
  });
}

function getOutputFiles(kind, name, artifacts) {
  const key = getArtifactName({ artifactType: kind, crateName: name });
  const outputFiles = artifacts[key];

  if (outputFiles) {
    return { key, outputFiles };
  }

  // Cargo started replacing `-` with `_` in artifact names. Reverse the process
  // and check again. https://github.com/rust-lang/cargo/issues/13867
  const altKey = key.replace(/_/g, "-");

  return {
    key: altKey,
    outputFiles: artifacts[altKey],
  };
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

  // Apple Silicon (M1, etc.) requires shared libraries to be signed. However,
  // the macOS code signing cache isn't cleared when overwriting a file.
  // Deleting the file before copying works around the issue.
  //
  // Unfortunately, this workaround is incomplete because the file must be
  // deleted from the location it is loaded. If further steps in the user's
  // build process copy or move the file in place, the code signing cache
  // will not be cleared.
  //
  // https://github.com/neon-bindings/neon/issues/911
  if (extname(outputFile) === ".node") {
    try {
      await unlink(outputFile);
    } catch (_e) {
      // Ignore errors; the file might not exist
    }
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

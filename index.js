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

	// TODO: Check if all this validation is necessary
	// https://crates.io/crates/cargo_metadata
	if (!data || data.reason !== "compiler-artifact" || !data.target) {
		return;
	}

	const { name } = data.target;
	const outputFile = options.outputFiles[name];

	if (!outputFile || !Array.isArray(data.filenames)) {
		return;
	}

	const [filename] = data.filenames;

	if (!filename) {
		return;
	}

	copyArtifact(filename, outputFile)
		.then(() => {
			copied[name] = true;
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
				"cargo-cp-artifct my-crate=index.node",
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

// Expects: List of "crate-name=output/file" pairs
function parseOutputFiles(args) {
	return args
		.map(parseOutputFile)
		.reduce((acc, [crate, file]) => ({
			...acc,
			[crate]: file
		}), {});
}

// Expects: "crate-name=output/file" pair
function parseOutputFile(pair) {
	const splitAt = pair.indexOf("=");

	if (splitAt < 0) {
		quitError(`Missing output file name: ${pair}`);
	}

	return [pair.slice(0, splitAt), pair.slice(splitAt + 1)];
}

function quitError(msg) {
	console.error(msg);
	process.exit(1);
}

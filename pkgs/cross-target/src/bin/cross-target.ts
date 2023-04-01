#!/usr/bin/env node

import { spawnSync } from 'child_process';
import finder from "find-package-json";
import handlebars from "handlebars";
import die from "../die";
import * as rust from '../rust';
import * as node from '../node';

function help() {
    console.error(`
usage:
    cross-target --current script [-- args ...]
        Run cross-target \`script\` from package.json for the
        current device's system target, with the specified
        command-line arguments.

    cross-target --all script [-- args ...]
        Run cross-target \`script\` from package.json for all
        targets specified in package.json, passing the specified
        command-line arguments to each invocation of the script.
`.trim());
    console.error();
}

async function main(args: string[]) {
    if (args.length < 2) {
        help();
        die("not enough arguments provided");
    }

    const targetsSpec = args.shift();

    if (targetsSpec !== "--current" && targetsSpec !== "--all") {
        help();
        die("expected '--current' or '--all'");
    }

    const scriptKey = args.shift()!;

    if (args.length > 0) {
        if (args.shift() !== "--") {
            help();
            die("expected '--'");
        }
    }

    const manifests = finder();
    const first = manifests.next();
    if (first.done) {
        help();
        die("no package.json found");
    }

    const manifest: finder.PackageWithPath = first.value;
    const crossTarget: unknown = manifest['cross-target'] ?? {};

    if (!isCrossTarget(crossTarget)) {
        help();
        die("invalid 'cross-target' specification");
    }

    const template = crossTarget.scripts?.[scriptKey];

    if (!template) {
        help();
        die(`script ${scriptKey} not found`);
    }

    const compiled = handlebars.compile(template, { noEscape: true });

    const targets = targetsSpec === "--current"
        ? [rust.Target.current()]
        : crossTarget.targets
        ? crossTarget.targets.map(rust.Target.parse)
        : DEFAULT_TARGETS.map(rust.Target.fromNode)

    for (const target of targets) {
        const metadata = target.templateMetadata();
        const script = compiled(metadata);
        console.error(`⚙️ cross-target: running "${script}" for target "${target.toString()}"`);
        const result = spawnSync(script, { stdio: "inherit", shell: true });
        if (result.status !== 0) {
            process.exit(result.status || 1);
        }
    }
}

const DEFAULT_TARGETS: node.Target[] = [
    new node.Target('x64', 'darwin'),
    new node.Target('x64', 'linux'),
    new node.Target('x64', 'win32')
];

// Expand to support other toolchains in the future (node-gyp? clang? gcc? zig?)
type Toolchain = "rust";

type Target = string;

type CrossTarget = {
    toolchain: Toolchain,
    targets?: Target[],
    scripts?: Record<string, string>,
};

// FIXME: use zod to clean this up
function isCrossTarget(value: unknown): value is CrossTarget {
    if (typeof value !== 'object' || !value) {
        return false;
    }

    // Currently the only supported toolchain.
    if (!('toolchain' in value) || (value.toolchain !== 'rust')) {
        return false;
    }

    if ('targets' in value) {
        const targets = value.targets;
        if (!Array.isArray(targets) || !targets.every(x => typeof x === 'string')) {
            return false;
        }
    }

    if ('scripts' in value) {
        const scripts = value.scripts;
        if (!(scripts instanceof Object)) {
            return false;
        }
        for (const key in scripts) {
            if (typeof (scripts as Record<string, unknown>)[key] !== 'string') {
                return false;
            }
        }
    }

    return true;
}

main(process.argv.slice(2));

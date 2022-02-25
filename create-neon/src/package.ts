import { promises as fs } from "fs";
import * as path from "path";
import versions from "../data/versions.json";
import shell from "./shell";

const KEYS = [
  "name",
  "version",
  "description",
  "main",
  "scripts",
  "author",
  "license",
];

function sort(json: any): any {
  // First copy the keys in the order specified in KEYS.
  let next = KEYS.filter((key) => json.hasOwnProperty(key))
    .map((key) => [key, json[key]])
    .reduce((acc, [key, val]) => Object.assign(acc, { [key]: val }), {});

  // Then copy any remaining keys in the original order.
  return Object.assign(next, json);
}

export default class Package {
  name: string;
  version: string;
  author: string;
  quotedAuthor: string;
  license: string;
  description: string;
  quotedDescription: string;

  static async create(name: string, dir: string): Promise<Package> {
    let seed = {
      name: name,
      version: "0.1.0",
      main: "index.node",
      scripts: {
        build:
          "cargo-cp-artifact -nc index.node -- cargo build --message-format=json-render-diagnostics",
        "build-debug": "npm run build --",
        "build-release": "npm run build -- --release",
        install: "npm run build-release",
        test: "cargo test",
      },
      devDependencies: {
        "cargo-cp-artifact": `^${versions["cargo-cp-artifact"]}`,
      },
    };

    let filename = path.join(dir, "package.json");

    // 1. Write initial values to prevent `npm init` from asking unnecessary questions.
    await fs.writeFile(filename, JSON.stringify(seed));

    // 2. Call `npm init` to ask the user remaining questions.
    await shell("npm", ["init"], dir);

    // 3. Sort the values in idiomatic `npm init` order.
    let sorted = sort(JSON.parse(await fs.readFile(filename, "utf8")));

    // 4. Save the result to package.json.
    await fs.writeFile(filename, JSON.stringify(sorted, undefined, 2));

    return new Package(sorted);
  }

  constructor(json: any) {
    this.name = json.name;
    this.version = json.version;
    this.author = json.author;
    this.quotedAuthor = JSON.stringify(json.author);
    this.license = json.license;
    this.description = json.description;
    this.quotedDescription = JSON.stringify(json.description);
  }
}

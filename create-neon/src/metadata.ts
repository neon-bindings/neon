import { readFile, writeFile } from "fs/promises";

export interface Metadata {
  project: Project,
  versions: Versions
}

export class Project {
  private json: any;

  name: string;
  version: string;
  author: FreeText | undefined;
  license: string;
  description: FreeText | undefined;

  static async load(source: string, cargoCpArtifact: string): Promise<Project> {
    return new Project(JSON.parse(await readFile(source, 'utf8')), cargoCpArtifact);
  }

  constructor(json: any, cargoCpArtifact: string) {
    this.json = json;
    this.name = json.name || "";
    this.version = json.version || "";
    this.author = quote(json.author);
    this.license = json.license || "";
    this.description = quote(json.description);

    json.name = this.name;
    json.version = this.version;
    json.author = this.author?.raw;
    json.license = this.license;
    json.description = this.description?.raw;

    json.main = "index.node";

    let test = "cargo test";

    // If the user specifies a non-default test command, use theirs instead.
    // Ideally there would be better extensibility hooks in `npm init` for
    // this but there aren't any environment variables or command-line flags
    // we can use, so we have to guess based on the default value. This also
    // unfortunately leaks to the user when `npm init` shows the values for
    // the package.json it's going to use in the final user confirmation.
    if (!/\s*echo \".*\" && exit 1\s*/.test(json.scripts.test)) {
      test = json.scripts.test;
    }

    json.scripts = {
      "build": "cargo-cp-artifact -nc index.node -- cargo build --message-format=json-render-diagnostics",
      "install": "npm run build",
      "test": test
    };

    json.devDependencies = {
      "cargo-cp-artifact": `^${cargoCpArtifact}`
    };
  }

  async save(target: string): Promise<null> {
    await writeFile(target, JSON.stringify(this.json, undefined, 2))
    return null;
  }
}

export interface FreeText {
  raw: string;
  quoted: string;
}

function quote(text: string): FreeText | undefined {
  if (!text) {
    return undefined;
  }

  return {
    raw: text,
    quoted: JSON.stringify(text)
  };
}

export interface Versions {
  neon: string,
  napi: number
}

import { readFile } from "fs/promises";

export interface Metadata {
  project: Project,
  versions: Versions
}

export interface Project {
  name: string;
  version: string;
  author?: FreeText;
  license: string;
  description?: FreeText;
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

export namespace Project {
  export async function load(source: string): Promise<Project> {
    let json = JSON.parse(await readFile(source, 'utf8'));
    return {
      name: json.name || "",
      version: json.version || "",
      author: quote(json.author),
      license: json.license || "",
      description: quote(json.description)
    };
  }
}

export interface Versions {
  neon: string,
  napi: number
}

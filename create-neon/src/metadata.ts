import Package from './package';
import versions from '../data/versions.json';

export interface Metadata {
  project: Project,
  versions: Versions
}

export namespace Metadata {
  export function from(pkg: Package): Metadata {
    // Select the N-API version associated with the current
    // running Node process.
    let inferred = process.versions.napi;

    let napi = inferred
      ? Math.min(Number(versions.napi), Number(inferred))
      : Number(versions.napi);

    return {
      project: project(pkg),
      versions: {
        neon: versions.neon,
        napi: napi
      }
    };
  }
}

export interface Project {
  name: string;
  version: string;
  author: FreeText | undefined;
  license: string;
  description: FreeText | undefined;
}

function project(pkg: Package): Project {
  return {
    name: pkg.name,
    version: pkg.version,
    author: quote(pkg.author),
    license: pkg.license,
    description: quote(pkg.description)
  };
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

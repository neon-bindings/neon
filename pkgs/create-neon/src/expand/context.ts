import { ProjectOptions } from "../create/creator.js";
import { VERSIONS, Versions } from "./versions.js";

export type PackageContext = {
  name: string,
  version: string,
  author: string,
  license: string,
  description: string
};

export type CrateContext = {
  name: string,
  version: string,
  description: string | undefined,
  author: string | undefined,
  license: string | undefined,
};

export class Context {
  options: ProjectOptions;
  package: PackageContext | undefined;
  crate: CrateContext | undefined;
  crateStrings: CrateContext | undefined;
  versions: Versions;

  constructor(options: ProjectOptions) {
    this.options = options;
    this.package = undefined;
    this.crate = undefined;
    this.crateStrings = undefined;
    this.versions = VERSIONS;
  }
}

import { ProjectOptions } from "../create/creator.js";
import { VERSIONS, Versions } from "./versions.js";

export type ManifestData = {
  name: string;
  version: string;
  description: string | undefined;
  author: string | undefined;
  license: string | undefined;
};

type CrateData = ManifestData & {
  // The same manifest data but escaped as string literals
  // so they can be embedded in TOML.
  escaped: ManifestData;
};

export class Context {
  options: ProjectOptions;
  package: ManifestData | undefined;
  crate: CrateData | undefined;
  versions: Versions;

  constructor(options: ProjectOptions) {
    this.options = options;
    this.package = undefined;
    this.crate = undefined;
    this.versions = VERSIONS;
  }
}

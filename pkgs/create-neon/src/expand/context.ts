import Package, { PackageOptions } from "../package.js";
import { VERSIONS, Versions } from "./versions.js";

export class Context {
  options: PackageOptions;
  package: Package | undefined;
  versions: Versions;

  constructor(options: PackageOptions) {
    this.options = options;
    this.package = undefined;
    this.versions = VERSIONS;
  }
}

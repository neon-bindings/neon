import { Cache } from "../cache.js";

export class NPM implements Cache {
  readonly org: string;
  readonly prefix: string;

  readonly type: string = "npm";

  constructor(org: string, prefix: string) {
    this.org = org;
    this.prefix = prefix;
  }
}

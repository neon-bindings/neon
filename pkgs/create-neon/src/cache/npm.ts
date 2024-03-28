import { Cache } from "../cache.js";

export class NPM implements Cache {
  readonly org: string | null;

  readonly type: string = "npm";

  constructor(org: string | null) {
    this.org = org;
  }
}

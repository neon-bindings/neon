import { Cache } from "../cache.js";

export class NPM implements Cache {
  readonly org: string;

  readonly type: string = "npm";

  constructor(pkg: string, org?: string) {
    this.org = org || NPM.inferOrg(pkg);
  }

  static inferOrg(pkg: string): string {
    const m = pkg.match(/^@([^/]+)\/(.*)/);
    return `@${m?.[1] ?? pkg}`;
  }
}

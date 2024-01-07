import { Cache } from '../cache.js';

export class NPM implements Cache {
  private _org: string | null;

  constructor(org: string | null) {
    this._org = org;
  }
}

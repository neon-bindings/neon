// This whole module is a bummer but was the best I could figure out since:
// - Using @sindresorhus packages like 'chalk' and 'execa' forces a project to use Node's native ESM support.
// - This means the tsconfig must generate a modern format like es2022.
// - When generating ESM, TS doesn't support importing JSON files with static typing without import assertions.
// - Import assertions are not yet stable in Node, and produce an instability warning.
//
// So for the time being, this module simply implements the static typing explicitly.
// If and when TS adds back and way to infer the static types when importing a JSON file
// and generates a stable format that Node doesn't complain about, we can eliminate this
// boilerplate wrapper module.

import { createRequire } from 'module';

export type Versions = {
  neon: string,
  "cargo-cp-artifact": string
};

const KEYS = ['neon', 'cargo-cp-artifact'];

function assertIsVersions(data: unknown): asserts data is Versions {
  if (!data || typeof data !== 'object') {
    throw new TypeError("expected object");
  }
  KEYS.forEach(key => {
    if (!(key in data)) {
      throw new TypeError(`require '${key}' property not found`);
    }
  });
}

const dynamicRequire = createRequire(import.meta.url);

function load(): Versions {
  const data = dynamicRequire('../data/versions.json');
  assertIsVersions(data);
  return data;
}

export const VERSIONS: Versions = load();

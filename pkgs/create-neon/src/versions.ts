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

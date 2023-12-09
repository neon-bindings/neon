import { createRequire } from 'module';

type VersionData = {
  "neon": string,
  "cargo-cp-artifact": string
};

function assertIsVersionData(data: unknown): asserts data is VersionData {
  if (!data || typeof data !== 'object') {
    throw new TypeError("expected object");
  }
  if (!('neon' in data)) {
    throw new TypeError("required 'neon' property not found");
  }
  if (!('cargo-cp-artifact' in data)) {
    throw new TypeError("require 'cargo-cp-artifact' property not found");
  }
}

const dynamicRequire = createRequire(import.meta.url);

function load(): VersionData {
  const data = dynamicRequire('../data/versions.json');
  assertIsVersionData(data);
  return data;
}

export const VERSIONS: VersionData = load();

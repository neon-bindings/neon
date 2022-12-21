const addon = require("..");

describe("Serde", () => {
  const suite = addon.build_serde_test_suite();

  for (const [k, v] of Object.entries(suite)) {
    it(k, v);
  }
});
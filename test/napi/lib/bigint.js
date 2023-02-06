const addon = require("..");

describe("JsBigInt", () => {
  const suite = addon.bigint_suite();

  for (const [k, v] of Object.entries(suite)) {
    it(k, v);
  }
});

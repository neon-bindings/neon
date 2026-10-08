const addon = require("..");
const { assert } = require("chai");

describe("Node-API 9", function () {
  it("should get a symbol from the global registry", function () {
    assert.strictEqual(addon.napi9SymbolFor("neon"), Symbol.for("neon"));
  });

  it("should get the module file name", function () {
    assert.match(addon.napi9ModuleFileName(), /^file:\/\/.*index\.node$/);
  });
});

describe("Node-API 10", function () {
  it("should create a property key", function () {
    assert.deepEqual(addon.napi10PropertyKey("hello", 42), { hello: 42 });
  });
});

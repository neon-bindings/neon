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

describe("property keys", function () {
  it("should get and set with property key handles and string keys", function () {
    const keys = ["a", "hello", "héllo", "🥹", "", "17"];
    const obj = addon.propertyKeyRoundtrip(keys, 42);

    assert.deepEqual(Object.keys(obj).sort(), [...keys].sort());
    for (const key of keys) {
      assert.strictEqual(obj[key], 42);
    }
  });

  it("should reuse a property key handle across objects", function () {
    const objs = [{ x: 1 }, { x: 2, y: 5 }, { y: 1, x: 3 }];

    assert.strictEqual(addon.sumByPropertyKey("x", objs), 6);
  });
});

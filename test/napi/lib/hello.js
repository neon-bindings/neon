var addon = require("..");
var assert = require("chai").assert;

describe("hello", function () {
  it("should export a greeting", function () {
    assert.strictEqual(addon.greeting, "Hello, World!");
    assert.strictEqual(addon.greeting, addon.greetingCopy);
  });

  it("should export global singletons for JS primitives", function () {
    assert.strictEqual(addon.undefined, undefined);
    assert.ok(addon.hasOwnProperty("undefined"));
    assert.strictEqual(addon.null, null);
    assert.strictEqual(addon.true, true);
    assert.strictEqual(addon.false, false);
  });

  it("should export numbers", function () {
    assert.strictEqual(addon.one, 1);
    assert.strictEqual(addon.two, 2.1);
  });

  it("should be able to create JS objects in rust", function () {
    assert.deepEqual(addon.rustCreated, {
      0: 1,
      a: 1,
      whatever: true,
    });
  });

  it("should export a Rust function", function () {
    assert.strictEqual(addon.add1(2), 3.0);
  });
});

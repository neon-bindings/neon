const addon = require("..");
const { expect } = require("chai");
const assert = require("chai").assert;

describe("container", function () {
  it("can produce and consume a RefCell", function () {
    const cell = addon.createStringRefCell("my sekret mesij");
    const s = addon.readStringRefCell(cell);
    assert.strictEqual(s, "my sekret mesij");
  });

  it("concatenates a RefCell<String> with a String", function () {
    const cell = addon.createStringRefCell("hello");
    const s = addon.stringRefCellConcat(cell, " world");
    assert.strictEqual(s, "hello world");
  });

  it("fails with a type error when not given a RefCell", function () {
    try {
      addon.stringRefCellConcat("hello", " world");
      assert.fail("should have thrown");
    } catch (err) {
      assert.instanceOf(err, TypeError);
      assert.strictEqual(err.message, "expected std::cell::RefCell");
    }
  });
});

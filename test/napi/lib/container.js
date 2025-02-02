const addon = require("..");
const { expect } = require("chai");
const assert = require("chai").assert;

describe("container", function () {
  it("can produce and consume a RefCell", function () {
    const cell = addon.createStringRefCell("my sekret mesij");
    const s = addon.readStringRefCell(cell);
    assert.strictEqual("my sekret mesij", s);
  });

  it("concatenates a RefCell<String> with a String", function () {
    const cell = addon.createStringRefCell("hello");
    const s = addon.stringRefCellConcat(cell, " world");
    assert.strictEqual("hello world", s);
  });
});

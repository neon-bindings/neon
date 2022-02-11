var addon = require("../native");
var assert = require("chai").assert;

describe("JsNumber", function () {
  it("return a JsNumber built in Rust", function () {
    assert.equal(addon.return_js_number(), 9000);
  });

  it("return a JsNumber for a large int built in Rust", function () {
    assert.equal(addon.return_large_js_number(), 4294967296);
  });

  it("return a negative JsNumber int built in Rust", function () {
    assert.equal(addon.return_negative_js_number(), -9000);
  });

  it("return a JsNumber float built in Rust", function () {
    assert.equal(addon.return_float_js_number(), 1.4747);
  });

  it("return a negative JsNumber float built in Rust", function () {
    assert.equal(addon.return_negative_float_js_number(), -1.4747);
  });

  describe("round trips", function () {
    it("accept and return a number", function () {
      assert.equal(addon.accept_and_return_js_number(1), 1);
    });

    it("accept and return a large number as a JsNumber", function () {
      assert.equal(
        addon.accept_and_return_large_js_number(9007199254740991),
        9007199254740991
      );
    });

    it("will be safe below Number.MAX_SAFE_INTEGER", function () {
      assert.notEqual(
        addon.accept_and_return_large_js_number(9007199254740990),
        9007199254740991
      );
    });

    it("will not be save above Number.MAX_SAFE_INTEGER", function () {
      assert.equal(
        addon.accept_and_return_large_js_number(9007199254740993),
        9007199254740992
      );
    });

    it("accept and return a float as a JsNumber", function () {
      assert.equal(addon.accept_and_return_float_js_number(0.23423), 0.23423);
    });

    it("accept and return a negative number as a JsNumber", function () {
      assert.equal(addon.accept_and_return_negative_js_number(-55), -55);
    });
  });
});

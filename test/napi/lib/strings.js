var addon = require("..");
var { assert, expect } = require("chai");

describe("JsString", function () {
  it("should return a JsString built in Rust", function () {
    assert.equal(addon.return_js_string(), "hello node");
  });
  it("should return a raw valid UTF-16 string built in Rust", function () {
    const decoder = new TextDecoder("utf-16");
    assert.equal(decoder.decode(addon.return_js_string_utf16()), "hello 🥹");
  });
  describe("encoding", function () {
    it("should return the UTF-8 string length", function () {
      assert.equal(addon.return_length_utf8("hello 🥹"), 10);
    });
    it("should return the UTF-16 string length", function () {
      assert.equal(addon.return_length_utf16("hello 🥹"), 8);
    });
  });
  describe("run_as_script", function () {
    it("should return the evaluated value", function () {
      assert.equal(addon.run_string_as_script("6 * 7"), 42);
    });
    it("should throw if the script throws", function () {
      expect(() =>
        addon.run_string_as_script('throw new Error("b1-66er")')
      ).to.throw("b1-66er");
    });
    it("should throw SyntaxError if the string has invalid syntax", function () {
      expect(() => addon.run_string_as_script("invalid js code")).to.throw(
        SyntaxError
      );
    });
  });
});

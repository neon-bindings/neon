var addon = require('..');
var { assert, expect } = require('chai');

describe('JsString', function() {
  it('should return a JsString built in Rust', function () {
    assert.equal(addon.return_js_string(), "hello node");
  });
  describe('run_as_script', function () {
    it('should return the evaluated value', function () {
      assert.equal(addon.run_string_as_script('6 * 7'), 42);
    });
    it('should throw if the script throws', function () {
      expect(() => addon.run_string_as_script('throw new Error("b1-66er")')).to.throw('b1-66er');
    });
  })
});

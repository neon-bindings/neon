var addon = require('../native');
var assert = require('chai').assert;

describe('JsNumber', function() {
  it('return a JsNumber built in Rust', function () {
    assert.equal(9000, addon.return_js_number());
  });

  it('return a JsNumber for a large int built in Rust', function () {
    assert.equal(4294967296, addon.return_large_js_number());
  });

  it('return a negative JsNumber int built in Rust', function () {
    assert.equal(-9000, addon.return_negative_js_number());
  });

  it('return a JsNumber float built in Rust', function () {
    assert.equal(1.4747, addon.return_float_js_number());
  });

  it('return a negative JsNumber float built in Rust', function () {
    assert.equal(-1.4747, addon.return_negative_float_js_number());
  });

  describe('round trips', function () {
    it('accept and return a number', function () {
      assert.equal(1, addon.accept_and_return_js_number(1));
    });

    it('accept and return a large number as a JsNumber', function () {
      assert.equal(9007199254740991, addon.accept_and_return_large_js_number(9007199254740991));
    });

    it('will be safe below Number.MAX_SAFE_INTEGER', function () {
      assert.notEqual(9007199254740991, addon.accept_and_return_large_js_number(9007199254740990));
    });

    it('will not be save above Number.MAX_SAFE_INTEGER', function () {
      assert.equal(9007199254740992, addon.accept_and_return_large_js_number(9007199254740993));
    });

    it('accept and return a float as a JsNumber', function () {
      assert.equal(0.23423, addon.accept_and_return_float_js_number(0.23423));
    });

    it('accept and return a negative number as a JsNumber', function () {
      assert.equal(-55, addon.accept_and_return_negative_js_number(-55));
    });
  });

});

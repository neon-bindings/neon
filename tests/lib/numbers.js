var addon = require('../native');
var assert = require('chai').assert;

describe('JsNumber', function() {
  it('should return a JsNumber built in Rust', function () {
    assert.equal(9000, addon.return_js_number());
  });

  it('should return a JsNumber for a large int built in Rust', function () {
    assert.equal(4294967296, addon.return_large_js_number());
  });

  it('should return a negative JsNumber int built in Rust', function () {
    assert.equal(-9000, addon.return_negative_js_number());
  });

  it('should return a JsNumber float built in Rust', function () {
    assert.equal(1.4747, addon.return_float_js_number());
  });

  it('should return a negative JsNumber float built in Rust', function () {
    assert.equal(-1.4747, addon.return_negative_float_js_number());
  });
});

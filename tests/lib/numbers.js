var addon = require('../native');
var assert = require('chai').assert;

describe('JsString', function() {
  it('should return a JsNumber built in Rust', function () {
    assert.equal(9000, addon.return_js_number());
  });

  it('should return a JsNumber for a large int built in Rust', function () {
    assert.equal(4294967296, addon.return_large_js_number());
  });

  it('should return a negative JsNumber int built in Rust', function () {
    assert.equal(-9000, addon.return_negative_js_number());
  });
});

var addon = require('../native');
var assert = require('chai').assert;

describe('JsArray', function() {
  it('return a JsArray built in Rust', function () {
    assert.deepEqual([], addon.return_js_array());
  });

  it('return a JsArray with an integer at index 0', function () {
    assert.deepEqual([9000], addon.return_js_array_with_integer());
  });

  it('return a JsArray with an string at index 0', function () {
    assert.deepEqual(["hello neon"], addon.return_js_array_with_string());
  });
});

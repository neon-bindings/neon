var addon = require('../native');
var assert = require('chai').assert;

describe('JsArray', function() {
  it('should return a JsArray built in Rust', function () {
    assert.deepEqual([], addon.return_js_array());
  });

  it('should return a JsArray with an integer at index 0', function () {
    assert.deepEqual([9000], addon.return_js_array_with_integer());
  });

  it('should return a JsArray with an string at index 0', function () {
    assert.deepEqual(["hello node"], addon.return_js_array_with_string());
  });
});

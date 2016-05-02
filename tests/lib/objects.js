var addon = require('../native');
var assert = require('chai').assert;

describe('JsObject', function() {
  it('should return a JsObject built in Rust', function () {
    assert.deepEqual({}, addon.return_js_object());
  });

  it('should return a JsObject with an integer key value pair', function () {
    assert.deepEqual({number: 9000}, addon.return_js_object_with_integer());
  });

  it('should return a JsObject with an string key value pair', function () {
    assert.deepEqual({string: "hello node"}, addon.return_js_object_with_string());
  });
});

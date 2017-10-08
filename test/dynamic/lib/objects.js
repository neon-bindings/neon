var addon = require('../native');
var assert = require('chai').assert;

describe('JsObject', function() {
  it('return the v8::Global object', function () {
      assert(global === addon.return_js_global_object());
  });

  it('return a JsObject built in Rust', function () {
    assert.deepEqual({}, addon.return_js_object());
  });

  it('return a JsObject with a number key value pair', function () {
    assert.deepEqual({number: 9000}, addon.return_js_object_with_number());
  });

  it('return a JsObject with an string key value pair', function () {
    assert.deepEqual({string: "hello node"}, addon.return_js_object_with_string());
  });

  it('return a JsObject with mixed content key value pairs', function () {
    assert.deepEqual({number: 9000, string: 'hello node'}, addon.return_js_object_with_mixed_content());
  });
});

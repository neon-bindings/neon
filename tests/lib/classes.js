var addon = require('../native');
var assert = require('chai').assert;

describe('JsClass', function() {
  it('return a JsClass built in Rust', function () {
    assert.isFunction(addon.return_js_class());
  });

  it('return a JsClass built in Rust that implements x => x + 1', function () {
    // assert.equal(addon.return_js_class()(41), 42);
  });

});

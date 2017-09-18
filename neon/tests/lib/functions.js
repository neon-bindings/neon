var addon = require('../native');
var assert = require('chai').assert;

describe('JsFunction', function() {
  it('return a JsFunction built in Rust', function () {
    assert.isFunction(addon.return_js_function());
  });

  it('return a JsFunction built in Rust that implements x => x + 1', function () {
    assert.equal(addon.return_js_function()(41), 42);
  });

  it('call a JsFunction built in JS that implements x => x + 1', function () {
    assert.equal(addon.call_js_function(function(x) { return x + 1 }), 17);
  });

  it('new a JsFunction', function () {
    assert.equal(addon.construct_js_function(Date), 1970);
  });

  it('got two parameters, a string and a number', function() {
    addon.check_string_and_number("string", 42);
  });

  it('converts a Rust panic to a throw in a function', function() {
    assert.throws(function() { addon.panic() }, Error, /^internal error in native module: zomg$/);
  });

  it('lets panic override a throw', function() {
    assert.throws(function() { addon.panic_after_throw() }, Error, /^internal error in native module: this should override the RangeError$/);
  });
});

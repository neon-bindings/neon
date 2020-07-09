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
    assert.throws(function() { addon.panic() }, Error, /^internal error in Neon module: zomg$/);
  });

  it('lets panic override a throw', function() {
    assert.throws(function() { addon.panic_after_throw() }, Error, /^internal error in Neon module: this should override the RangeError$/);
  });

  it('catches an excption with cx.try_catch', function() {
    var error = new Error('Something bad happened');
    assert.equal(addon.throw_and_catch(error), error);
    assert.equal(addon.throw_and_catch(42), 42);
    assert.equal(addon.throw_and_catch('a string'), 'a string');
    assert.equal(addon.call_and_catch(() => { throw 'shade' }), 'shade');
    assert.equal(addon.call_and_catch(() => {
      throw addon.call_and_catch(() => {
        throw addon.call_and_catch(() => {
          throw 'once';
        }) + ' upon';
      }) + ' a';
    }) + ' time', 'once upon a time');
  });

  it('computes the right number of arguments', function() {
    assert.equal(addon.num_arguments(), 0);
    assert.equal(addon.num_arguments('a'), 1);
    assert.equal(addon.num_arguments('a', 'b'), 2);
    assert.equal(addon.num_arguments('a', 'b', 'c'), 3);
    assert.equal(addon.num_arguments('a', 'b', 'c', 'd'), 4);
  });

  it('gets the right `this`-value', function() {
    var o = { iamobject: 'i am object' };
    assert.equal(addon.return_this.call(o), o);

    var d = new Date();
    assert.equal(addon.return_this.call(d), d);

    var n = 19;
    assert.notStrictEqual(addon.return_this.call(n), n);
  });

  it('can manipulate an object `this` binding', function() {
    var o = { modified: false };
    addon.require_object_this.call(o);
    assert.equal(o.modified, true);
    // Doesn't throw because of implicit primitive wrapping:
    addon.require_object_this.call(42);
  });

  it('implicitly gets global', function() {
    var global = (new Function("return this"))();
    assert.equal(addon.return_this.call(undefined), global);
  });

  it('exposes an argument via arguments_opt iff it is there', function() {
    assert.equal(addon.is_argument_zero_some(), false);
    assert.equal(addon.is_argument_zero_some('a'), true);
    assert.equal(addon.is_argument_zero_some('a', 'b'), true);
    assert.equal(addon.is_argument_zero_some.call(null), false);
    assert.equal(addon.is_argument_zero_some.call(null, ['a']), true);
    assert.equal(addon.is_argument_zero_some.call(null, ['a', 'b']), true);
  });

  it('correctly casts an argument via cx.arguments', function() {
    assert.equal(addon.require_argument_zero_string('foobar'), 'foobar');
    assert.throws(function() { addon.require_argument_zero_string(new Date()) }, TypeError);
    assert.throws(function() { addon.require_argument_zero_string(17) }, TypeError);
  });

  it('executes a scoped computation', function() {
    assert.equal(addon.execute_scoped(), 99);
  });

  it('computes a value in a scoped computation', function() {
    assert.equal(addon.compute_scoped(), 99);
  });
});

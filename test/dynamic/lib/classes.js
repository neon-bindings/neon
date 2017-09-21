var addon = require('../native');
const User = addon.User;
const PanickyAllocator = addon.PanickyAllocator;
const PanickyConstructor = addon.PanickyConstructor;
var assert = require('chai').assert;

describe('JsClass', function() {
  it('return a JsClass built in Rust', function () {
    assert.isFunction(addon.User);
  });

  it('return a JsClass built in Rust', function () {
    var u = new User(1, "some", "thing", "else");
    assert(u instanceof User);
  });

  it('can use getter funtion defined in Rust', function () {
    var u = new User(1, "some", "thing", "else");
    assert.equal(u.get('id'), 1);
    assert.equal(u.get('first_name'), "some");
    assert.equal(u.get('last_name'), "thing");
    assert.equal(u.get('email'), "else");
    assert.throw(function() { u.get('not_a_field') }, TypeError);
  });

  it('converts a Rust panic to a throw in a method', function() {
    var u = new User(1, "some", "thing", "else");
    assert.throws(function() { u.panic() }, Error, /^internal error in native module: User.prototype.panic$/);
  });

  it('converts a Rust panic to a throw in a constructor call', function() {
    assert.throws(function() { PanickyConstructor() }, Error, /^internal error in native module: constructor call panicking$/);
  });

  it('converts a Rust panic to a throw in a constructor new', function() {
    assert.throws(function() { new PanickyConstructor() }, Error, /^internal error in native module: constructor panicking$/);
  });

  it('converts a Rust panic to a throw in a constructor allocator', function() {
    assert.throws(function() { new PanickyAllocator() }, Error, /^internal error in native module: allocator panicking$/);
  });
});

var addon = require('../native');
const User = addon.User;
var assert = require('chai').assert;

describe('JsClass', function() {
  it('return a JsClass built in Rust', function () {
    assert.isFunction(addon.User);
  });

  it('return a JsClass built in Rust', function () {
    u = new User(1, "some", "thing", "else");
    assert(u instanceof User);
  });

  it('can use getter funtion defined in Rust', function () {
    u = new User(1, "some", "thing", "else");
    assert.equal(u.get('id'), 1);
    assert.equal(u.get('first_name'), "some");
    assert.equal(u.get('last_name'), "thing");
    assert.equal(u.get('email'), "else");
    assert.throw(function() { u.get('not_a_field') }, TypeError);
  });
});

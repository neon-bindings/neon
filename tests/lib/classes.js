var addon = require('../native');
const User = addon.User;
var assert = require('chai').assert;

describe('JsClass', function() {
  it('return a JsClass built in Rust', function () {
    assert.isFunction(addon.User);
  });

  it('return a JsClass built in Rust', function () {
    console.log(User.toString());
    u = User(1, "some", "thing", "else")
    assert.equal(user, 9);
  });

});

const addon = require('../native');
const assert = require('chai').assert;

describe('errors', function() {
  it('should be able to create an error', function () {
    const msg = "Oh, no!";
    const err = addon.new_error(msg);
    
    assert.instanceOf(err, Error);
    assert.strictEqual(err.message, msg);
  });

  it('should be able to create a type error', function () {
    const msg = "Type error? From Rust?!";
    const err = addon.new_type_error(msg);
    
    assert.instanceOf(err, TypeError);
    assert.instanceOf(err, Error);
    assert.strictEqual(err.message, msg);
  });

  it('should be able to create a range error', function () {
    const msg = "Out of Bounds";
    const err = addon.new_range_error(msg);
    
    assert.instanceOf(err, RangeError);
    assert.instanceOf(err, Error);
    assert.strictEqual(err.message, msg);
  });

  it('should be able to throw an error', function () {
    const msg = "Out of Bounds";
    
    assert.throws(() => addon.throw_error(msg), msg);
  });

  it('should be able to stringify a downcast error', function () {
    let msg = addon.downcast_error();
    assert.strictEqual(msg, "failed to downcast string to number");
  });

});

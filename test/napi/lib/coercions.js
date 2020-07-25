var addon = require('../native');
var assert = require('chai').assert;

describe('coercions', function() {
  it('can stringify', function () {
    assert.strictEqual(addon.to_string([1, 2, 3]), '1,2,3');
    assert.strictEqual(addon.to_string(new Map()), '[object Map]');
    assert.strictEqual(addon.to_string({ a: 'b' }), '[object Object]');
  });
});

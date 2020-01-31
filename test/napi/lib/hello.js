var addon = require('../native');
var assert = require('chai').assert;

describe('hello', function() {
  it('should export a greeting', function () {
    assert.strictEqual(addon.greeting, "Hello, World!");
    assert.strictEqual(addon.greeting, addon.greetingCopy);
  });

  it('should export global singletons for JS primitives', function () {
    assert.strictEqual(addon.undefined, undefined);
    assert.ok(addon.hasOwnProperty('undefined'));
    assert.strictEqual(addon.null, null);
    assert.strictEqual(addon.true, true);
    assert.strictEqual(addon.false, false);
  });
});

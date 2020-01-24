var addon = require('../native');
var assert = require('chai').assert;

describe('hello', function() {
  it('should export a greeting', function () {
    assert.equal(addon.greeting, "Hello, World!");
    assert.equal(addon.greeting, addon.greetingCopy);
  });

  it('should export global singletons for JS primitives', function () {
    assert.equal(addon.undefined, undefined);
    assert.equal(addon.null, null);
  });
});

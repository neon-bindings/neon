var addon = require('../native');
var assert = require('chai').assert;

describe('hello', function() {
  it('should export a greeting', function () {
    assert.equal(addon.greeting, "Hello, World!");
  });
});

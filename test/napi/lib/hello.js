var addon = require('../native');
var assert = require('chai').assert;

describe('hello', function() {
  it('should load the module without crashing', function () {
    assert.isTrue(true, "loaded native module without crashing");
  });
});

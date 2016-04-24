var addon = require('../native');
var assert = require('chai').assert;

describe('JsString', function() {
  it('should return a JsInteger built in Rust', function () {
    assert.equal(9000, addon.return_js_integer());
  });
});

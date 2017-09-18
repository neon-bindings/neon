var addon = require('../native');
var assert = require('chai').assert;

describe('JsString', function() {
  it('should return a JsString built in Rust', function () {
    assert.equal(addon.return_js_string(), "hello node");
  });
});

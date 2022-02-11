var addon = require("..");
var assert = require("chai").assert;

describe("type checks", function () {
  it("is_array", function () {
    assert(addon.is_array([]));
    assert(addon.is_array(new Array()));
    assert(!addon.is_array(null));
    assert(!addon.is_array(1));
    assert(!addon.is_array({ 0: "a", 1: "b", length: 2 }));
  });

  it("is_array_buffer", function () {
    assert(addon.is_array_buffer(new ArrayBuffer(0)));
    assert(!addon.is_array_buffer(new DataView(new ArrayBuffer(0))));
    assert(!addon.is_array_buffer(new Uint8Array(1024)));
    assert(!addon.is_array_buffer(Buffer.alloc(64)));
    assert(!addon.is_array_buffer([]));
    assert(!addon.is_array_buffer("hello world"));
  });

  it("is_boolean", function () {
    assert(addon.is_boolean(true));
    assert(addon.is_boolean(false));
    assert(!addon.is_boolean(new Boolean(true)));
    assert(!addon.is_boolean(new Boolean(false)));
  });

  it("is_buffer", function () {
    assert(addon.is_buffer(Buffer.alloc(64)));
    assert(addon.is_buffer(new Uint8Array(64)));
    assert(!addon.is_buffer(new ArrayBuffer(64)));
  });

  it("is_error", function () {
    assert(addon.is_error(new Error()));
    assert(addon.is_error(new TypeError()));
    class SubclassError extends Error {}
    assert(addon.is_error(new SubclassError()));
    assert(!addon.is_error("something went wrong!"));
  });

  it("is_null", function () {
    assert(addon.is_null(null));
    assert(!addon.is_null(undefined));
    assert(!addon.is_null("anything other than null"));
  });

  it("is_number", function () {
    assert(addon.is_number(0));
    assert(addon.is_number(1.4526456453));
    assert(addon.is_number(NaN));
    assert(!addon.is_number(new Number(2)));
    assert(!addon.is_number("42"));
  });

  it("is_object", function () {
    assert(addon.is_object({}));
    assert(addon.is_object(new Number(1)));
    assert(addon.is_object(new String("1")));
    // Unlike `typeof`, is_object does *not* consider `null` to be an Object.
    assert(!addon.is_object(null));
    assert(!addon.is_object(undefined));
    assert(!addon.is_object(1));
    assert(!addon.is_object("1"));
  });

  it("is_string", function () {
    assert(addon.is_string("1"));
    assert(!addon.is_string(new String("1")));
  });

  it("is_undefined", function () {
    assert(addon.is_undefined(undefined));
    assert(!addon.is_undefined(null));
    assert(!addon.is_undefined("anything other than undefined"));
  });

  it("strict_equals", function () {
    assert(addon.strict_equals(17, 17));
    assert(!addon.strict_equals(17, 18));
    let o1 = {};
    let o2 = {};
    assert(addon.strict_equals(o1, o1));
    assert(!addon.strict_equals(o1, o2));
    assert(!addon.strict_equals(o1, 17));
  });
});

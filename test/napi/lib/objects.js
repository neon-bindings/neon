var addon = require("..");
var assert = require("chai").assert;

describe("JsObject", function () {
  it("return the v8::Global object", function () {
    assert(global === addon.return_js_global_object());
  });

  it("return a JsObject built in Rust", function () {
    assert.deepEqual({}, addon.return_js_object());
  });

  it("return a JsObject with a number key value pair", function () {
    assert.deepEqual({ number: 9000 }, addon.return_js_object_with_number());
  });

  it("return a JsObject with an string key value pair", function () {
    assert.deepEqual(
      { string: "hello node" },
      addon.return_js_object_with_string()
    );
  });

  it("return a JsObject with mixed content key value pairs", function () {
    assert.deepEqual(
      { number: 9000, string: "hello node" },
      addon.return_js_object_with_mixed_content()
    );
  });

  it("freeze a JsObject", function () {
    const originalValue = 1;
    const obj = { x: originalValue };

    assert.doesNotThrow(function () {
      addon.freeze_js_object(obj);
    }, "freeze_js_object should not throw");

    obj.x = 2;
    assert.equal(
      obj.x,
      originalValue,
      "freeze_js_object should not allow mutation"
    );

    const shouldNotFreeze = new Uint32Array(8);
    assert.throws(function () {
      addon.freeze_js_object(shouldNotFreeze);
    });
  });

  it("seal a JsObject", function () {
    const obj = { x: 1 };

    assert.doesNotThrow(function () {
      addon.seal_js_object(obj);
    }, "seal_js_object should not throw");

    delete obj.x;
    assert.isOk(obj.x, "seal_js_object should not allow property deletion");

    const shouldNotSeal = new Uint32Array(8);
    assert.throws(function () {
      addon.freeze_js_object(shouldNotSeal);
    });
  });

  it("returns only own properties from get_own_property_names", function () {
    var superObject = {
      a: 1,
    };

    var childObject = Object.create(superObject);
    childObject.b = 2;

    assert.deepEqual(
      addon.get_own_property_names(childObject),
      Object.getOwnPropertyNames(childObject)
    );
  });

  it("does not return Symbols from get_own_property_names", function () {
    var object = {};
    object["this should be a thing"] = 0;
    object[Symbol("this should not be a thing")] = 1;

    assert.deepEqual(
      addon.get_own_property_names(object),
      Object.getOwnPropertyNames(object)
    );
    assert.equal(addon.get_own_property_names(object).length, 1);
  });

  it("data borrowed on the heap can be held longer than the handle", function () {
    const msg = "Hello, World!";
    const buf = Buffer.from(msg);

    assert.strictEqual(addon.byte_length(msg), buf.length);
    assert.strictEqual(addon.byte_length(buf), buf.length);
  });

  it("calling Object::call_with() properly calls object methods", function () {
    const obj = {
      value: 42,
      nullary() {
        return this.value;
      },
      unary(x) {
        return this.value + x;
      },
    };

    assert.strictEqual(addon.call_nullary_method(obj), 42);
    assert.strictEqual(addon.call_unary_method(obj, 17), 59);
  });

  it("calling Object::call_with() with a symbol method name works", function () {
    const sym = Symbol.for("mySymbol");
    const obj = {
      [sym]() {
        return "hello";
      },
    };

    assert.strictEqual(addon.call_symbol_method(obj, sym), "hello");
  });

  it("extracts an object property with .prop()", function () {
    const obj = { number: 3.141593 };

    assert.strictEqual(addon.get_property_with_prop(obj), 3.141593);
  });

  it("sets an object property with .prop()", function () {
    const obj = { number: 3.141593 };

    addon.set_property_with_prop(obj);

    assert.strictEqual(obj.number, 42);
  });

  it("calls a method with .prop()", function () {
    const obj = {
      name: "Diana Prince",
      setName(name) {
        this.name = name;
      },
      toString() {
        return `[object ${this.name}]`;
      },
    };

    assert.strictEqual(obj.toString(), "[object Diana Prince]");
    assert.strictEqual(
      addon.call_methods_with_prop(obj),
      "[object Wonder Woman]"
    );
    assert.strictEqual(obj.toString(), "[object Wonder Woman]");
  });

  it("throws a TypeError when calling a non-method with .prop()", function () {
    const obj = {
      number: 42,
    };

    assert.throws(() => {
      addon.call_non_method_with_prop(obj);
    }, /failed to downcast/);
  });
});

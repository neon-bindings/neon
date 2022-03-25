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

  it("correctly reads a TypedArray using the borrow API", function () {
    var b = new ArrayBuffer(32);
    var a = new Int32Array(b, 4, 4);
    a[0] = 49;
    a[1] = 1350;
    a[2] = 11;
    a[3] = 237;
    assert.equal(addon.read_typed_array_with_borrow(a, 0), 49);
    assert.equal(addon.read_typed_array_with_borrow(a, 1), 1350);
    assert.equal(addon.read_typed_array_with_borrow(a, 2), 11);
    assert.equal(addon.read_typed_array_with_borrow(a, 3), 237);
  });

  it("correctly writes to a TypedArray using the borrow_mut API", function () {
    var b = new ArrayBuffer(32);
    var a = new Int32Array(b, 4, 4);
    addon.write_typed_array_with_borrow_mut(a, 0, 43);
    assert.equal(a[0], 43);
    addon.write_typed_array_with_borrow_mut(a, 1, 1000);
    assert.equal(a[1], 1000);
    addon.write_typed_array_with_borrow_mut(a, 2, 22);
    assert.equal(a[2], 22);
    addon.write_typed_array_with_borrow_mut(a, 3, 243);
    assert.equal(a[3], 243);
  });

  it("correctly reads a Buffer as a typed array", function () {
    var a = Buffer.from([49, 135, 11, 237]);
    assert.equal(addon.read_u8_typed_array(a, 0), 49);
    assert.equal(addon.read_u8_typed_array(a, 1), 135);
    assert.equal(addon.read_u8_typed_array(a, 2), 11);
    assert.equal(addon.read_u8_typed_array(a, 3), 237);
  });

  it("copies the contents of one typed array to another", function () {
    const a = new Uint32Array([1, 2, 3, 4]);
    const b = new Uint32Array(a.length);

    addon.copy_typed_array(a, b);

    assert.deepEqual([...a], [...b]);
  });

  it("cannot borrow overlapping buffers", function () {
    const buf = new ArrayBuffer(20);
    const arr = new Uint32Array(buf);
    const a = new Uint32Array(buf, 4, 2);
    const b = new Uint32Array(buf, 8, 2);

    assert.throws(() => addon.copy_typed_array(a, b));
  });

  it("gets a 16-byte, zeroed ArrayBuffer", function () {
    var b = addon.return_array_buffer();
    assert.equal(b.byteLength, 16);
    assert.equal(new Uint32Array(b)[0], 0);
    assert.equal(new Uint32Array(b)[1], 0);
    assert.equal(new Uint32Array(b)[2], 0);
    assert.equal(new Uint32Array(b)[3], 0);
  });

  it("correctly reads an ArrayBuffer using the lock API", function () {
    var b = new ArrayBuffer(16);
    var a = new Uint32Array(b);
    a[0] = 47;
    a[1] = 133;
    a[2] = 9;
    a[3] = 88888888;
    assert.equal(addon.read_array_buffer_with_lock(a, 0), 47);
    assert.equal(addon.read_array_buffer_with_lock(a, 1), 133);
    assert.equal(addon.read_array_buffer_with_lock(a, 2), 9);
    assert.equal(addon.read_array_buffer_with_lock(a, 3), 88888888);
  });

  it("correctly reads an ArrayBuffer using the borrow API", function () {
    var b = new ArrayBuffer(4);
    var a = new Uint8Array(b);
    a[0] = 49;
    a[1] = 135;
    a[2] = 11;
    a[3] = 237;
    assert.equal(addon.read_array_buffer_with_borrow(b, 0), 49);
    assert.equal(addon.read_array_buffer_with_borrow(b, 1), 135);
    assert.equal(addon.read_array_buffer_with_borrow(b, 2), 11);
    assert.equal(addon.read_array_buffer_with_borrow(b, 3), 237);
  });

  it("correctly writes to an ArrayBuffer using the lock API", function () {
    var b = new ArrayBuffer(16);
    addon.write_array_buffer_with_lock(b, 0, 3);
    assert.equal(new Uint8Array(b)[0], 3);
    addon.write_array_buffer_with_lock(b, 1, 42);
    assert.equal(new Uint8Array(b)[1], 42);
    addon.write_array_buffer_with_lock(b, 2, 127);
    assert.equal(new Uint8Array(b)[2], 127);
    addon.write_array_buffer_with_lock(b, 3, 255);
    assert.equal(new Uint8Array(b)[3], 255);
  });

  it("correctly writes to an ArrayBuffer using the borrow_mut API", function () {
    var b = new ArrayBuffer(4);
    addon.write_array_buffer_with_borrow_mut(b, 0, 43);
    assert.equal(new Uint8Array(b)[0], 43);
    addon.write_array_buffer_with_borrow_mut(b, 1, 100);
    assert.equal(new Uint8Array(b)[1], 100);
    addon.write_array_buffer_with_borrow_mut(b, 2, 22);
    assert.equal(new Uint8Array(b)[2], 22);
    addon.write_array_buffer_with_borrow_mut(b, 3, 243);
    assert.equal(new Uint8Array(b)[3], 243);
  });

  it("gets a 16-byte, uninitialized Buffer", function () {
    var b = addon.return_uninitialized_buffer();
    assert.ok(b.length === 16);
  });

  it("gets a 16-byte, zeroed Buffer", function () {
    var b = addon.return_buffer();
    assert.ok(b.equals(Buffer.alloc(16)));
  });

  it("gets an external Buffer", function () {
    var expected = "String to copy";
    var buf = addon.return_external_buffer(expected);
    assert.instanceOf(buf, Buffer);
    assert.strictEqual(buf.toString(), expected);
  });

  it("gets an external ArrayBuffer", function () {
    var expected = "String to copy";
    var buf = addon.return_external_array_buffer(expected);
    assert.instanceOf(buf, ArrayBuffer);
    assert.strictEqual(Buffer.from(buf).toString(), expected);
  });

  it("correctly reads a Buffer using the lock API", function () {
    var b = Buffer.allocUnsafe(16);
    b.writeUInt8(147, 0);
    b.writeUInt8(113, 1);
    b.writeUInt8(109, 2);
    b.writeUInt8(189, 3);
    assert.equal(addon.read_buffer_with_lock(b, 0), 147);
    assert.equal(addon.read_buffer_with_lock(b, 1), 113);
    assert.equal(addon.read_buffer_with_lock(b, 2), 109);
    assert.equal(addon.read_buffer_with_lock(b, 3), 189);
  });

  it("correctly reads a Buffer using the borrow API", function () {
    var b = Buffer.from([149, 224, 70, 229]);
    assert.equal(addon.read_buffer_with_borrow(b, 0), 149);
    assert.equal(addon.read_buffer_with_borrow(b, 1), 224);
    assert.equal(addon.read_buffer_with_borrow(b, 2), 70);
    assert.equal(addon.read_buffer_with_borrow(b, 3), 229);
  });

  it("correctly writes to a Buffer using the lock API", function () {
    var b = Buffer.allocUnsafe(16);
    b.fill(0);
    addon.write_buffer_with_lock(b, 0, 6);
    assert.equal(b.readUInt8(0), 6);
    addon.write_buffer_with_lock(b, 1, 61);
    assert.equal(b.readUInt8(1), 61);
    addon.write_buffer_with_lock(b, 2, 45);
    assert.equal(b.readUInt8(2), 45);
    addon.write_buffer_with_lock(b, 3, 216);
    assert.equal(b.readUInt8(3), 216);
  });

  it("correctly writes to a Buffer using the borrow_mut API", function () {
    var b = Buffer.alloc(4);
    addon.write_buffer_with_borrow_mut(b, 0, 16);
    assert.equal(b[0], 16);
    addon.write_buffer_with_borrow_mut(b, 1, 100);
    assert.equal(b[1], 100);
    addon.write_buffer_with_borrow_mut(b, 2, 232);
    assert.equal(b[2], 232);
    addon.write_buffer_with_borrow_mut(b, 3, 55);
    assert.equal(b[3], 55);
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
});

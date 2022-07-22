var addon = require("..");
var assert = require("chai").assert;

const { Worker, isMainThread, parentPort } = require('worker_threads');

if (!isMainThread) {
  parentPort.on('message', (message) => {
    // transfer it back
    parentPort.postMessage(message, [message]);
  });

  return;
}

// A background thread we can transfer buffers to as a way to force
// them to be detached (see the `detach` function).
const DETACH_WORKER = new Worker(__filename);

// Allow the test harness to spin down the background thread and exit
// when the main thread completes.
DETACH_WORKER.unref();

function detach(buffer) {
  if (!(buffer instanceof ArrayBuffer)) {
    throw new TypeError();
  }

  DETACH_WORKER.postMessage(buffer, [buffer]);

  let resolve, reject;

  let promise = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });

  DETACH_WORKER.once('message', (message) => {
    resolve(message);
  });

  return promise;
};

describe("Typed arrays", function () {
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

  it("gets a typed array constructed from an ArrayBuffer", function () {
    var b = new ArrayBuffer(64);
    var i8 = addon.return_int8array_from_arraybuffer(b);
    assert.strictEqual(i8.byteLength, 64);
    assert.strictEqual(i8.length, 64);
    i8[0] = 0x17;
    i8[1] = -0x17;
    assert.deepEqual([...i8.slice(0, 2)], [0x17, -0x17]);

    var b = new ArrayBuffer(64);
    var i16 = addon.return_int16array_from_arraybuffer(b);
    assert.strictEqual(i16.byteLength, 64);
    assert.strictEqual(i16.length, 32);
    i16[0] = 0x1234;
    i16[1] = -1;
    i16[2] = -2;
    i16[3] = 0x5678;
    assert.deepEqual([...i16.slice(0, 4)], [0x1234, -1, -2, 0x5678]);
    var u8 = new Uint8Array(b);
    assert.deepEqual(
      [...u8.slice(0, 8)],
      [0x34, 0x12, 0xff, 0xff, 0xfe, 0xff, 0x78, 0x56]
    );

    var b = new ArrayBuffer(64);
    var u32 = addon.return_uint32array_from_arraybuffer(b);
    assert.strictEqual(u32.byteLength, 64);
    assert.strictEqual(u32.length, 16);
    u32[0] = 0x12345678;
    var u8 = new Uint8Array(b);
    assert.deepEqual([...u8.slice(0, 4)], [0x78, 0x56, 0x34, 0x12]);

    var b = new ArrayBuffer(64);
    var f64 = addon.return_float64array_from_arraybuffer(b);
    assert.strictEqual(f64.byteLength, 64);
    assert.strictEqual(f64.length, 8);
    f64[0] = 1.0;
    f64[1] = 2.0;
    f64[2] = 3.141592653589793;
    assert.deepEqual([...f64.slice(0, 3)], [1.0, 2.0, 3.141592653589793]);
    assert.deepEqual(
      [...new Float64Array(b).slice(0, 3)],
      [1.0, 2.0, 3.141592653589793]
    );

    var b = new ArrayBuffer(64);
    var u64 = addon.return_biguint64array_from_arraybuffer(b);
    assert.strictEqual(u64.byteLength, 64);
    assert.strictEqual(u64.length, 8);
    u64[0] = 0x1234567887654321n;
    u64[1] = 0xcafed00d1337c0den;
    var u8 = new Uint8Array(b);
    assert.deepEqual(
      [...u64.slice(0, 2)],
      [0x1234567887654321n, 0xcafed00d1337c0den]
    );
    assert.deepEqual(
      [...u8.slice(0, 16)],
      [
        0x21, 0x43, 0x65, 0x87, 0x78, 0x56, 0x34, 0x12, 0xde, 0xc0, 0x37, 0x13,
        0x0d, 0xd0, 0xfe, 0xca,
      ]
    );
  });

  it("gets a new typed array", function () {
    var i32 = addon.return_new_int32array(16);
    assert.strictEqual(i32.constructor, Int32Array);
    assert.strictEqual(i32.byteLength, 64);
    assert.strictEqual(i32.length, 16);
    assert.deepEqual(
      [...i32],
      [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
  });

  it("gets correct typed array info", function () {
    var buf = new ArrayBuffer(128);

    var a = addon.return_int8array_from_arraybuffer(buf);
    var info = addon.get_typed_array_info(a);

    assert.strictEqual(buf, a.buffer);
    assert.strictEqual(0, a.byteOffset);
    assert.strictEqual(128, a.length);
    assert.strictEqual(128, a.byteLength);

    assert.strictEqual(buf, info.buffer);
    assert.strictEqual(0, info.byteOffset);
    assert.strictEqual(128, info.length);
    assert.strictEqual(128, info.byteLength);

    var a = addon.return_int16array_from_arraybuffer(buf);
    var info = addon.get_typed_array_info(a);

    assert.strictEqual(buf, a.buffer);
    assert.strictEqual(0, a.byteOffset);
    assert.strictEqual(64, a.length);
    assert.strictEqual(128, a.byteLength);

    assert.strictEqual(buf, info.buffer);
    assert.strictEqual(0, info.byteOffset);
    assert.strictEqual(64, info.length);
    assert.strictEqual(128, info.byteLength);

    var a = addon.return_uint32array_from_arraybuffer(buf);
    var info = addon.get_typed_array_info(a);

    assert.strictEqual(buf, a.buffer);
    assert.strictEqual(0, a.byteOffset);
    assert.strictEqual(32, a.length);
    assert.strictEqual(128, a.byteLength);

    assert.strictEqual(buf, info.buffer);
    assert.strictEqual(0, info.byteOffset);
    assert.strictEqual(32, info.length);
    assert.strictEqual(128, info.byteLength);

    var a = addon.return_biguint64array_from_arraybuffer(buf);
    var info = addon.get_typed_array_info(a);

    assert.strictEqual(buf, a.buffer);
    assert.strictEqual(0, a.byteOffset);
    assert.strictEqual(16, a.length);
    assert.strictEqual(128, a.byteLength);

    assert.strictEqual(buf, info.buffer);
    assert.strictEqual(0, info.byteOffset);
    assert.strictEqual(16, info.length);
    assert.strictEqual(128, info.byteLength);
  });

  it("correctly constructs a view over a slice of a buffer", function () {
    var buf = new ArrayBuffer(128);

    var a = addon.return_uint32array_from_arraybuffer_region(buf, 16, 4);
    var info = addon.get_typed_array_info(a);

    assert.strictEqual(buf, a.buffer);
    assert.strictEqual(16, a.byteOffset);
    assert.strictEqual(4, a.length);
    assert.strictEqual(16, a.byteLength);

    assert.strictEqual(buf, info.buffer);
    assert.strictEqual(16, info.byteOffset);
    assert.strictEqual(4, info.length);
    assert.strictEqual(16, info.byteLength);

    a[0] = 17;
    a[1] = 42;
    a[2] = 100;
    a[3] = 1000;

    var left = buf.slice(0, 16);
    var middle = buf.slice(16, 32);
    var right = buf.slice(32);

    assert.deepEqual(new Uint8Array(16), new Uint8Array(left));
    assert.deepEqual(
      new Uint8Array([17, 0, 0, 0, 42, 0, 0, 0, 100, 0, 0, 0, 232, 3, 0, 0]),
      new Uint8Array(middle)
    );
    assert.deepEqual(new Uint8Array(96), new Uint8Array(right));
  });

  it("properly fails to construct typed arrays with invalid arguments", function () {
    var buf = new ArrayBuffer(32);
    try {
      addon.return_uint32array_from_arraybuffer_region(buf, 1, 4);
      assert.fail("should have thrown for unaligned offset");
    } catch (expected) {}

    try {
      addon.return_uint32array_from_arraybuffer_region(buf, 100, 104);
      assert.fail("should have thrown for bounds check failure");
    } catch (expected) {}

    try {
      addon.return_uint32array_from_arraybuffer_region(buf, 0, 5);
      assert.fail("should have thrown for invalid length");
    } catch (expected) {}

    try {
      addon.return_uint32array_from_arraybuffer_region(buf, 0, 10);
      assert.fail("should have thrown for excessive length");
    } catch (expected) {}
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

  it("zeroes the byteLength when an ArrayBuffer is detached", function () {
    var buf = new ArrayBuffer(16);
    assert.strictEqual(buf.byteLength, 16);
    assert.strictEqual(addon.get_arraybuffer_byte_length(buf), 16);

    detach(buf);

    assert.strictEqual(buf.byteLength, 0);
    assert.strictEqual(addon.get_arraybuffer_byte_length(buf), 0);
  });

  it("provides correct metadata when detaching a typed array's buffer", function () {
    var buf = new ArrayBuffer(16);
    var arr = new Uint32Array(buf);

    assert.strictEqual(buf.byteLength, 16);
    assert.strictEqual(arr.byteLength, 16);
    assert.strictEqual(arr.length, 4);
    assert.strictEqual(addon.get_typed_array_info(arr).byteLength, 16);
    assert.strictEqual(addon.get_typed_array_info(arr).length, 4);
    assert.strictEqual(addon.get_typed_array_info(arr).buffer, buf);

    let { before, after } = addon.detach_same_handle(arr, (arr) => detach(arr.buffer));

    assert.strictEqual(before.byteLength, 16);
    assert.strictEqual(before.length, 4);

    assert.strictEqual(arr.byteLength, 0);
    assert.strictEqual(arr.length, 0);
    assert.strictEqual(addon.get_typed_array_info(arr).byteLength, 0);
    assert.strictEqual(addon.get_typed_array_info(arr).length, 0);

    assert.strictEqual(after.byteLength, 0);
    assert.strictEqual(after.length, 0);
  });

  it("provides correct metadata when detaching an escaped typed array's buffer", function () {
    let { before, after } = addon.detach_and_escape((arr) => detach(arr.buffer));
    assert.strictEqual(before.byteLength, 16);
    assert.strictEqual(before.length, 4);
    assert.strictEqual(after.byteLength, 0);
    assert.strictEqual(after.length, 0);
  });

  it("provides correct metadata when detaching a casted typed array's buffer", function () {
    var buf = new ArrayBuffer(16);
    var arr = new Uint32Array(buf);

    let { before, after } = addon.detach_and_cast(arr, (arr) => detach(arr.buffer));

    assert.strictEqual(before.byteLength, 16);
    assert.strictEqual(before.length, 4);
    assert.strictEqual(after.byteLength, 0);
    assert.strictEqual(after.length, 0);
  });

  it("provides correct metadata when detaching an un-rooted typed array's buffer", function () {
    var buf = new ArrayBuffer(16);
    var arr = new Uint32Array(buf);

    let { before, after } = addon.detach_and_unroot(arr, (arr) => detach(arr.buffer));

    assert.strictEqual(before.byteLength, 16);
    assert.strictEqual(before.length, 4);
    assert.strictEqual(after.byteLength, 0);
    assert.strictEqual(after.length, 0);
  });
});

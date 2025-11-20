const addon = require("..");
const assert = require("chai").assert;

describe("wrapping", function () {
  it("should be able to wrap a Rust value in an object", () => {
    const msg = "Hello, World!";
    const o = {};

    addon.wrapString(o, msg);
    assert.strictEqual(addon.unwrapString(o), msg);
  });

  it("should not be able to wrap an object twice", () => {
    const o = {};

    addon.wrapString(o, "Hello, World!");
    assert.throws(
      () => addon.wrapString(o, "nope"),
      /non-class instance expected/
    );
  });

  it("should not be able to unwrap an object that was not wrapped", () => {
    const o = {};

    assert.throws(() => addon.unwrapString(o), /class instance expected/);
  });
});

describe("classes", function () {
  it("can create a Message class", function () {
    const Message = addon.Message;

    const message = new Message("Hello, Neon, this is your test speaking.");
    assert.instanceOf(message, Message);
    assert.strictEqual(
      message.read(),
      "Hello, Neon, this is your test speaking."
    );
    const message2 = message.concat(new Message("  <<FNORD>>"));
    assert.strictEqual(
      message2.read(),
      "Hello, Neon, this is your test speaking.  <<FNORD>>"
    );
  });

  it("can mutate a Message with &mut self", function () {
    const Message = addon.Message;

    const message = new Message("Hello");
    assert.strictEqual(message.read(), "Hello");
    message.append(", World");
    assert.strictEqual(message.read(), "Hello, World");
    message.append("!");
    assert.strictEqual(message.read(), "Hello, World!");
  });

  it("can subclass a Neon class", function () {
    const Message = addon.Message;

    class LoudMessage extends Message {
      shout() {
        return this.read().toUpperCase();
      }
    }

    const message = new LoudMessage("Hello, Neon, this is your test speaking.");
    assert.instanceOf(message, Message);
    assert.instanceOf(message, LoudMessage);
    assert.strictEqual(
      message.read(),
      "Hello, Neon, this is your test speaking."
    );
    assert.strictEqual(
      message.shout(),
      "HELLO, NEON, THIS IS YOUR TEST SPEAKING."
    );
    const message2 = message.concat(new Message("  <<FNORD>>"));
    assert.strictEqual(
      message2.read(),
      "Hello, Neon, this is your test speaking.  <<FNORD>>"
    );
    assert.throws(() => message2.shout());
  });

  it("can create a Point class", function () {
    const Point = addon.Point;

    const point = new Point(1, 2);
    assert.instanceOf(point, Point);
    assert.strictEqual(point.x(), 1);
    assert.strictEqual(point.y(), 2);

    const point2 = new Point(3, 4);
    assert.instanceOf(point2, Point);
    assert.strictEqual(point2.x(), 3);
    assert.strictEqual(point2.y(), 4);
    assert.strictEqual(point.distance(point2), Math.sqrt(8));
  });

  it("fails with a TypeError when passing a non-object as &Point argument", function () {
    const Point = addon.Point;

    const point = new Point(1, 2);
    assert.instanceOf(point, Point);
    assert.strictEqual(point.x(), 1);
    assert.strictEqual(point.y(), 2);

    assert.throws(() => {
      point.distance(42);
    }, TypeError);
  });

  it("can mutate a Point with &mut self", function () {
    const Point = addon.Point;

    const point = new Point(10, 20);
    assert.strictEqual(point.x(), 10);
    assert.strictEqual(point.y(), 20);

    point.moveBy(5, 3);
    assert.strictEqual(point.x(), 15);
    assert.strictEqual(point.y(), 23);

    point.setX(100);
    assert.strictEqual(point.x(), 100);
    assert.strictEqual(point.y(), 23);

    point.setY(200);
    assert.strictEqual(point.x(), 100);
    assert.strictEqual(point.y(), 200);
  });

  it("can swap coordinates between two Points using &mut references", function () {
    const Point = addon.Point;

    const p1 = new Point(10, 20);
    const p2 = new Point(30, 40);

    assert.strictEqual(p1.x(), 10);
    assert.strictEqual(p1.y(), 20);
    assert.strictEqual(p2.x(), 30);
    assert.strictEqual(p2.y(), 40);

    p1.swapCoords(p2);

    assert.strictEqual(p1.x(), 30);
    assert.strictEqual(p1.y(), 40);
    assert.strictEqual(p2.x(), 10);
    assert.strictEqual(p2.y(), 20);
  });

  it("fails with a TypeError when passing a non-object as &mut Point argument", function () {
    const Point = addon.Point;

    const point = new Point(1, 2);
    assert.instanceOf(point, Point);
    assert.strictEqual(point.x(), 1);
    assert.strictEqual(point.y(), 2);

    assert.throws(() => {
      point.swapCoords(42);
    }, TypeError);
  });

  it("fails with TypeError when passing wrong type to Message.concat", function () {
    const Message = addon.Message;
    const Point = addon.Point;
    const message = new Message("Hello");
    const point = new Point(1, 2);

    // Test with various wrong types
    assert.throws(
      () => message.concat(null),
      TypeError,
      /expected object/,
      "should reject null"
    );
    assert.throws(
      () => message.concat(undefined),
      TypeError,
      /expected object/,
      "should reject undefined"
    );
    assert.throws(
      () => message.concat("string"),
      TypeError,
      /expected object/,
      "should reject string"
    );
    assert.throws(
      () => message.concat(42),
      TypeError,
      /expected object/,
      "should reject number"
    );
    assert.throws(
      () => message.concat({ value: "test" }),
      TypeError,
      /class instance expected/,
      "should reject plain object"
    );
    assert.throws(
      () => message.concat([]),
      TypeError,
      /class instance expected/,
      "should reject array"
    );
    assert.throws(
      () => message.concat(point),
      TypeError,
      /expected instance of.*Message/,
      "should reject instance of different class"
    );
  });

  it("fails with TypeError when passing wrong type to Point.midpoint", function () {
    const Point = addon.Point;
    const point = new Point(5, 10);

    // Test with various wrong types
    assert.throws(
      () => point.midpoint(null),
      TypeError,
      /expected object/,
      "should reject null"
    );
    assert.throws(
      () => point.midpoint(undefined),
      TypeError,
      /expected object/,
      "should reject undefined"
    );
    assert.throws(
      () => point.midpoint("string"),
      TypeError,
      /expected object/,
      "should reject string"
    );
    assert.throws(
      () => point.midpoint(123),
      TypeError,
      /expected object/,
      "should reject number"
    );
    assert.throws(
      () => point.midpoint({ x: 1, y: 2 }),
      TypeError,
      /class instance expected/,
      "should reject plain object"
    );
  });

  it("fails with TypeError when mixing different class types", function () {
    const Point = addon.Point;
    const Message = addon.Message;

    const point = new Point(1, 2);
    const message = new Message("test");

    // Try to pass a Message where a Point is expected
    try {
      point.distance(message);
      assert.fail("should have thrown an error");
    } catch (e) {
      assert.instanceOf(e, TypeError);
      assert.match(e.message, /expected instance of.*Point/);
      // Uncomment to see the actual error message:
      // console.log("Point error:", e.message);
    }

    // Try to pass a Point where a Message is expected
    try {
      message.concat(point);
      assert.fail("should have thrown an error");
    } catch (e) {
      assert.instanceOf(e, TypeError);
      assert.match(e.message, /expected instance of.*Message/);
      // Uncomment to see the actual error message:
      // console.log("Message error:", e.message);
    }
  });

  it("Point class has const properties", function () {
    const Point = addon.Point;

    // Test basic const properties
    assert.strictEqual(Point.ORIGIN_X, 0);
    assert.strictEqual(Point.ORIGIN_Y, 0);

    // Test const property with custom name
    assert.strictEqual(Point.maxCoordinate, 1000);

    // Test const property with JSON serialization
    assert.deepEqual(Point.DEFAULT_MESSAGE, ["hello", "point"]);
  });

  it("Point const properties are immutable", function () {
    const Point = addon.Point;

    // Store original values
    const originalX = Point.ORIGIN_X;
    const originalMaxCoord = Point.maxCoordinate;

    // Attempt to modify properties (should silently fail in non-strict mode)
    Point.ORIGIN_X = 999;
    Point.maxCoordinate = 5000;

    // Values should be unchanged
    assert.strictEqual(Point.ORIGIN_X, originalX);
    assert.strictEqual(Point.maxCoordinate, originalMaxCoord);

    // Check property descriptors
    const descX = Object.getOwnPropertyDescriptor(Point, "ORIGIN_X");
    assert.strictEqual(descX.writable, false);
    assert.strictEqual(descX.configurable, false);
    assert.strictEqual(descX.enumerable, true);

    const descMaxCoord = Object.getOwnPropertyDescriptor(
      Point,
      "maxCoordinate"
    );
    assert.strictEqual(descMaxCoord.writable, false);
    assert.strictEqual(descMaxCoord.configurable, false);
    assert.strictEqual(descMaxCoord.enumerable, true);
  });

  it("Point supports complex const expressions", function () {
    const Point = addon.Point;

    // Test computed const expressions
    assert.strictEqual(Point.COMPUTED_VALUE, 42); // 10 + 20 + 12
    assert.strictEqual(Point.SIZE_OF_F64, 8); // std::mem::size_of::<f64>()
    assert.strictEqual(Point.STRING_LENGTH, 7); // "complex".len()

    // Verify they're immutable
    const original = Point.COMPUTED_VALUE;
    Point.COMPUTED_VALUE = 999;
    assert.strictEqual(Point.COMPUTED_VALUE, original);
  });

  it("Point supports edge case const expressions", function () {
    const Point = addon.Point;

    // Test boolean const
    assert.strictEqual(Point.IS_2D, true);
    assert.strictEqual(typeof Point.IS_2D, "boolean");

    // Test conditional const expression
    assert.strictEqual(Point.MAX_DIMENSION, 2147483647);

    // Test match expression const
    assert.strictEqual(Point.COORDINATE_BYTES, 4);

    // Test const with arithmetic
    assert.strictEqual(Point.DOUBLE_100_SQUARED, 20000); // 100^2 * 2

    // Test string with special characters (renamed property)
    assert.strictEqual(Point.specialString, 'Hello\nWorld\t"quoted"\r\n');

    // Test negative number
    assert.strictEqual(Point.NEGATIVE_OFFSET, -42);

    // Test large integer (approximate)
    assert.strictEqual(Point.MAX_SAFE_INTEGER_APPROX, 2147483647);

    // Test const starting with underscore
    assert.strictEqual(Point._PRIVATE_CONST, 999);

    // Verify all edge case properties are immutable
    const props = [
      "IS_2D",
      "MAX_DIMENSION",
      "COORDINATE_BYTES",
      "DOUBLE_100_SQUARED",
      "specialString",
      "NEGATIVE_OFFSET",
      "MAX_SAFE_INTEGER_APPROX",
      "_PRIVATE_CONST",
    ];

    props.forEach((prop) => {
      const descriptor = Object.getOwnPropertyDescriptor(Point, prop);
      assert.strictEqual(
        descriptor.writable,
        false,
        `${prop} should not be writable`
      );
      assert.strictEqual(
        descriptor.configurable,
        false,
        `${prop} should not be configurable`
      );
      assert.strictEqual(
        descriptor.enumerable,
        true,
        `${prop} should be enumerable`
      );
    });
  });

  it("can create a StringBuffer class with Default constructor", function () {
    const StringBuffer = addon.StringBuffer;

    const buffer = new StringBuffer();
    buffer.push("Hello");
    buffer.push(" ");
    buffer.push("World");
    assert.strictEqual(buffer.toString(), "Hello World");
  });

  it("can use a renamed method in StringBuffer", function () {
    const StringBuffer = addon.StringBuffer;

    const buffer = new StringBuffer();
    buffer.push("  Hello  ");
    assert.strictEqual(buffer.trimStart(), "Hello  ");
    assert.strictEqual(buffer.trimEnd(), "  Hello");

    assert.isTrue(buffer.includes("Hello"));
    assert.isFalse(buffer.includes("World"));
  });

  // async tests
  it("AsyncClass should create instance", function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("hello");
    assert.ok(instance instanceof AsyncClass);
  });

  it("AsyncClass should have sync method", function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("hello");
    assert.strictEqual(instance.syncMethod(), "hello");
  });

  it("AsyncClass should have async method", async function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("hello");
    const result = await instance.asyncMethod(" world");
    assert.strictEqual(result, "hello world");
  });

  it("AsyncClass should have task method for CPU-intensive work", async function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    const result = await instance.heavyComputation();
    // Sum of 0..99 = (99 * 100) / 2 = 4950
    assert.strictEqual(result, 4950);
  });

  it("AsyncClass should have JSON method", function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    const input = ["item1", "item2", "item3"];
    const result = instance.jsonMethod(input);

    assert.strictEqual(typeof result, "object");
    assert.strictEqual(result.class_value, "test");
    assert.strictEqual(result.input_count, "3");
    assert.strictEqual(result.first_item, "item1");
  });

  it("AsyncClass should have explicit async method (Meta::Async)", async function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    const result = await instance.explicitAsyncMethod(3);
    assert.strictEqual(result, "Processing: test * 3");
  });

  it("AsyncClass should have explicit async method with clone", async function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    const result = await instance.explicitAsyncClone(" cloned");
    assert.strictEqual(result, "test cloned");
  });

  it("AsyncClass should have method with context parameter", function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("hello");
    const result = instance.methodWithContext(3);
    // "hello".length = 5, so 5 * 3 = 15
    assert.strictEqual(result, 15);
  });

  it("AsyncClass should have method with explicit context attribute", function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    const result = instance.methodWithExplicitContext(" suffix");
    assert.strictEqual(result, "test: suffix");
  });

  it("AsyncClass should have task method with channel parameter", async function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    const result = await instance.taskWithChannel(5);
    assert.strictEqual(result, "Task with channel: test * 5");
  });

  it("AsyncClass should have async fn method with channel parameter", async function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    const result = await instance.asyncFnWithChannel(" async");
    assert.strictEqual(result, "AsyncFn with channel: test async");
  });

  it("AsyncClass should have method with this parameter", function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    const result = instance.methodWithThis(" data");
    assert.strictEqual(
      result,
      "Instance: test, JS object available, data:  data"
    );
  });

  it("AsyncClass should have method with explicit this attribute", function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    const result = instance.methodWithExplicitThis(" suffix");
    assert.strictEqual(result, "Explicit this: test suffix");
  });

  it("AsyncClass should have method with context and this", function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("hello");
    const result = instance.methodWithContextAndThis(4);
    // "hello".length = 5, so 5 * 4 = 20
    assert.strictEqual(result, 20);
  });

  it("AsyncClass should have reasonable performance for simple methods", function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    const start = process.hrtime.bigint();

    // Run 1000 simple method calls
    for (let i = 0; i < 1000; i++) {
      instance.simpleMethod(i);
    }

    const end = process.hrtime.bigint();
    const durationMs = Number(end - start) / 1000000; // Convert to milliseconds

    // Should complete 1000 calls in reasonable time (less than 100ms is very good)
    assert(
      durationMs < 1000,
      `Performance test took ${durationMs}ms for 1000 calls`
    );
  });

  it("AsyncClass should handle JSON methods efficiently", function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    const testData = [1, 2, 3, 4, 5];

    const start = process.hrtime.bigint();

    // Run 100 JSON method calls
    for (let i = 0; i < 100; i++) {
      const result = instance.jsonMethodPerf(testData);
      assert.deepStrictEqual(result, [2, 4, 6, 8, 10]);
    }

    const end = process.hrtime.bigint();
    const durationMs = Number(end - start) / 1000000;

    // JSON serialization has overhead but should still be reasonable
    assert(
      durationMs < 1000,
      `JSON performance test took ${durationMs}ms for 100 calls`
    );
  });

  it("AsyncClass should handle context methods efficiently", function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");

    const start = process.hrtime.bigint();

    // Run 1000 context method calls
    for (let i = 0; i < 1000; i++) {
      const result = instance.contextMethodPerf(i);
      assert.strictEqual(result, i * 3);
    }

    const end = process.hrtime.bigint();
    const durationMs = Number(end - start) / 1000000;

    // Context methods should have minimal overhead
    assert(
      durationMs < 1000,
      `Context performance test took ${durationMs}ms for 1000 calls`
    );
  });

  it("AsyncClass async method consumes self", async function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    await instance.asyncMethod(" once");

    // This should fail because the instance has been consumed
    // In practice, we might want to document this limitation
    // or find a better approach for async methods
  });

  it("AsyncClass has const properties", function () {
    const AsyncClass = addon.AsyncClass;

    // Test basic const property
    assert.strictEqual(AsyncClass.DEFAULT_TIMEOUT, 5000);

    // Test const property with custom name and JSON serialization
    assert.deepEqual(AsyncClass.version, [1, 0, 0]);
  });

  it("AsyncClass should have explicit async JSON method", async function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    const input = [1, 2, 3, 4, 5];
    const result = await instance.explicitAsyncJsonMethod(input);

    assert.deepStrictEqual(result, [2, 4, 6, 8, 10]);
  });

  it("AsyncClass should have auto-detected async JSON method", async function () {
    const AsyncClass = addon.AsyncClass;
    const instance = new AsyncClass("test");
    const input = [1, 2, 3, 4, 5];
    const result = await instance.asyncJsonMethod(input);

    // Should multiply each element by 2
    assert.deepStrictEqual(result, [2, 4, 6, 8, 10]);
  });

  it("can create Point instances from Rust code", function () {
    const Point = addon.Point;

    // Test creating instance via Rust function (Rust → JS path)
    const rustPoint = addon.createPointFromRust(10, 20);

    // Verify it's an instance of the class
    assert.instanceOf(rustPoint, Point);
    assert.strictEqual(rustPoint.x(), 10);
    assert.strictEqual(rustPoint.y(), 20);

    // Test that Rust-created and JS-created instances are compatible
    const jsPoint = new Point(5, 10);
    const distance = rustPoint.distance(jsPoint);

    // Distance between (10, 20) and (5, 10) = sqrt((10-5)^2 + (20-10)^2) = sqrt(25 + 100) = sqrt(125)
    assert.strictEqual(distance, Math.sqrt(125));
  });

  it("can create origin Point from Rust", function () {
    const Point = addon.Point;

    const origin = addon.createPointOrigin();
    assert.instanceOf(origin, Point);
    assert.strictEqual(origin.x(), 0);
    assert.strictEqual(origin.y(), 0);
  });

  it("can transform Points in Rust and return new instances", function () {
    const Point = addon.Point;

    // Create point in JS
    const original = new Point(3, 4);

    // Pass to Rust, transform, and get back new instance
    const doubled = addon.doublePointCoords(original);

    // Verify the returned point
    assert.instanceOf(doubled, Point);
    assert.strictEqual(doubled.x(), 6);
    assert.strictEqual(doubled.y(), 8);

    // Verify original is unchanged
    assert.strictEqual(original.x(), 3);
    assert.strictEqual(original.y(), 4);
  });
});

describe("constructor features", function () {
  const { FallibleCounter, ContextCounter, JsonConfig, ValidatedConfig, Argv } =
    addon;

  describe("Result return types", function () {
    it("should create instance when constructor succeeds", () => {
      const counter = new FallibleCounter(50);
      assert.strictEqual(counter.get(), 50);
      counter.increment();
      assert.strictEqual(counter.get(), 51);
    });

    it("should throw error when constructor fails", () => {
      assert.throws(() => new FallibleCounter(150), /Value must be <= 100/);
    });

    it("should work with edge case: maximum valid value", () => {
      const counter = new FallibleCounter(100);
      assert.strictEqual(counter.get(), 100);
    });
  });

  describe("Context parameter support", function () {
    it("should create instance with context parameter", () => {
      const counter = new ContextCounter(42);
      assert.strictEqual(counter.get(), 42);
    });

    it("should work with different values", () => {
      const counter1 = new ContextCounter(10);
      const counter2 = new ContextCounter(20);
      assert.strictEqual(counter1.get(), 10);
      assert.strictEqual(counter2.get(), 20);
    });
  });

  describe("JSON support", function () {
    it("should create instance from JSON object", () => {
      const config = new JsonConfig({
        name: "test",
        count: 5,
        enabled: true,
      });
      assert.strictEqual(config.name(), "test");
      assert.strictEqual(config.count(), 5);
      assert.strictEqual(config.enabled(), true);
    });

    it("should work with different JSON values", () => {
      const config = new JsonConfig({
        name: "another",
        count: 100,
        enabled: false,
      });
      assert.strictEqual(config.name(), "another");
      assert.strictEqual(config.count(), 100);
      assert.strictEqual(config.enabled(), false);
    });
  });

  describe("Combined features: context + JSON + Result", function () {
    it("should create an instance with JSON array", () => {
      const argv = new Argv(["1", "2", "3", "4", "5"]);
      assert.strictEqual(argv.len(), 5);
      assert.strictEqual(argv.get(0), "1");
      assert.strictEqual(argv.get(4), "5");
    });

    it("should create instance with empty JSON array", () => {
      const argv = new Argv([]);
      assert.strictEqual(argv.len(), 0);
    });

    it("should create instance with nullable JSON array", () => {
      const argv = new Argv(null);
      assert.strictEqual(argv.len(), process.argv.length);
      for (let i = 0; i < process.argv.length; i++) {
        assert.strictEqual(argv.get(i), process.argv[i]);
      }
    });

    it("should propagate errors with nullable JSON array", () => {
      const old = Object.getOwnPropertyDescriptor(global, "process");
      try {
        // Temporarily override process to simulate error
        Object.defineProperty(global, "process", {
          get: () => {
            throw new Error("Temporarily broken");
          },
          set: () => {
            throw new Error("Temporarily broken");
          },
          enumerable: false,
          configurable: true,
        });
        assert.throws(() => {
          try {
            console.error("[ARGV TEST] getting process in JS");
            const process = global.process;
            console.error("[ARGV TEST] getting argv in JS");
            const argv = process.argv;
            console.error("[ARGV TEST] argv:", argv);
          } catch (e) {
            console.error("[ARGV TEST] Caught error in JS: ", e.message);
          }

          console.error("[ARGV TEST] now attempting in Rust");

          try {
            const { isMainThread } = require("worker_threads");
            console.error("[ARGV TEST] isMainThread:", isMainThread);
            return new Argv(null);
          } catch (e) {
            console.error("[ARGV TEST] Caught error: ", e.message);
            throw e;
          }
        }, /Temporarily broken/);
      } finally {
        // Restore original process object
        Object.defineProperty(global, "process", old);
      }
    });

    it("should create instance when validation passes", () => {
      const config = new ValidatedConfig({
        name: "valid",
        count: 500,
      });
      assert.strictEqual(config.name(), "valid");
      assert.strictEqual(config.count(), 500);
    });

    it("should throw error when name is empty", () => {
      assert.throws(
        () => new ValidatedConfig({ name: "", count: 10 }),
        /Name cannot be empty/
      );
    });

    it("should throw error when count is too large", () => {
      assert.throws(
        () => new ValidatedConfig({ name: "test", count: 2000 }),
        /Count must be <= 1000/
      );
    });

    it("should work with edge case: maximum valid count", () => {
      const config = new ValidatedConfig({
        name: "edge",
        count: 1000,
      });
      assert.strictEqual(config.count(), 1000);
    });
  });
});

const addon = require("..");
const { expect } = require("chai");
const assert = require("chai").assert;

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
});
